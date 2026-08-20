//! The native store: a compact JSONL cache plus an in-memory inverted index.
//!
//! Layout under the data directory:
//! ```text
//! <data_dir>/store/kjva.jsonl     — KJV Apocrypha verses
//! <data_dir>/store/lxx.jsonl      — Greek LXX (Apocrypha) verses
//! <data_dir>/meta/<dataset>.json  — provenance/licensing record
//! ```
//!
//! The store is built once by the importers and read on every invocation;
//! for an Apocrypha-sized corpus (~10k verses) loading and indexing is
//! single-digit milliseconds in release mode.

use std::collections::HashMap;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::domain::book::BookId;
use crate::domain::passage::{
    BookInfo, Chapter, DatasetInfo, Passage, Provenance, ScriptureText, Token,
};
use crate::domain::reference::{ChapterNumber, PassageRef, VerseNumber};
use crate::domain::search::{SearchHit, SearchQuery};
use crate::domain::source::ScriptureSource;
use crate::domain::witness::WitnessId;
use crate::error::{Result, ScribeError};
use crate::infrastructure::lemma_index::LemmaIndex;
use crate::text::normalize;

/// On-disk row format (short keys keep the cache small).
#[derive(Serialize, Deserialize, Clone)]
pub struct Row {
    pub b: String,
    pub c: u16,
    pub v: u16,
    pub t: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tok: Option<Vec<TokRow>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TokRow {
    pub s: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub l: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub m: Option<String>,
}

impl Row {
    pub fn into_domain(self, witness: WitnessId) -> Result<ScriptureText> {
        let book = crate::domain::book::resolve_book(&self.b)
            .map_err(|e| ScribeError::StoreCorrupt {
                path: "store".into(),
                detail: format!("unknown book {:?}: {e}", self.b),
            })?
            .0;
        let tokens = self
            .tok
            .map(|rows| {
                rows.into_iter()
                    .map(|r| Token {
                        normalized: r.n.unwrap_or_else(|| normalize(&r.s)),
                        surface: r.s,
                        lemma: r.l,
                        morph: r.m,
                    })
                    .collect()
            })
            .unwrap_or_else(|| {
                crate::text::tokenize(&self.t)
                    .into_iter()
                    .map(|n| Token {
                        surface: n.clone(),
                        normalized: n,
                        lemma: None,
                        morph: None,
                    })
                    .collect()
            });
        Ok(ScriptureText {
            witness,
            book,
            chapter: ChapterNumber::new(self.c),
            verse: VerseNumber::new(self.v),
            text: self.t,
            tokens,
        })
    }
}

/// In-memory Scripture source over the JSONL store.
pub struct Store {
    data_dir: PathBuf,
    verses: Vec<ScriptureText>,
    /// (witness, book, chapter) -> verse slice into `verses`.
    by_ref: HashMap<(WitnessId, BookId, u16), Range<u32>>,
    /// normalized token -> verse indices (sorted by construction).
    index: HashMap<String, Vec<u32>>,
    /// verse counts per witness.
    counts: HashMap<WitnessId, u64>,
    /// Greek lemma index, built lazily on the first word-study query so that
    /// passage/search commands never pay for it.
    lemma_index: OnceLock<LemmaIndex>,
}

impl Store {
    pub fn open(data_dir: &Path) -> Result<Store> {
        let mut store = Store {
            data_dir: data_dir.to_path_buf(),
            verses: Vec::new(),
            by_ref: HashMap::new(),
            index: HashMap::new(),
            counts: HashMap::new(),
            lemma_index: OnceLock::new(),
        };
        for witness in WitnessId::ALL {
            store.load_witness(witness)?;
        }
        Ok(store)
    }

    fn load_witness(&mut self, witness: WitnessId) -> Result<()> {
        let path = self.data_dir.join("store").join(witness.store_file());
        if !path.exists() {
            return Ok(());
        }
        let cache_path = self.data_dir.join("store").join(witness.cache_file());
        if let Some(verses) = crate::infrastructure::cache::read(&cache_path, witness)? {
            for verse in verses {
                self.index_verse(witness, verse);
            }
            return Ok(());
        }
        let content = std::fs::read_to_string(&path).map_err(|e| ScribeError::Io {
            path: path.display().to_string(),
            source: e,
        })?;
        let mut count = 0u64;
        for (lineno, line) in content.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let row: Row = serde_json::from_str(line).map_err(|e| ScribeError::StoreCorrupt {
                path: path.display().to_string(),
                detail: format!("line {}: {e}", lineno + 1),
            })?;
            let verse = row.into_domain(witness)?;
            self.index_verse(witness, verse);
            count += 1;
        }
        self.counts.insert(witness, count);
        Ok(())
    }

    /// Add one verse to the in-memory structures (by-ref ranges + token index).
    fn index_verse(&mut self, witness: WitnessId, verse: ScriptureText) {
        let idx = self.verses.len() as u32;
        self.by_ref
            .entry((witness, verse.book, verse.chapter.get()))
            .or_insert_with(|| idx..idx);
        self.by_ref
            .get_mut(&(witness, verse.book, verse.chapter.get()))
            .unwrap()
            .end = idx + 1;
        for tok in &verse.tokens {
            let postings = self.index.entry(tok.normalized.clone()).or_default();
            // Deduplicate: several occurrences of the same token in one
            // verse must not produce duplicate postings (they are
            // consecutive because verses are loaded in order).
            if postings.last() != Some(&idx) {
                postings.push(idx);
            }
        }
        *self.counts.entry(witness).or_insert(0) += 1;
        self.verses.push(verse);
    }

    pub fn count_verses(&self, witness: WitnessId) -> u64 {
        self.counts.get(&witness).copied().unwrap_or(0)
    }

    /// Is the Greek corpus installed (and non-empty)?
    pub fn greek_installed(&self) -> bool {
        self.counts.get(&WitnessId::Lxx).copied().unwrap_or(0) > 0
    }

    /// `scribe word <query>` — resolve to a lemma study.
    pub fn word_study(&self, query: &str) -> Result<crate::domain::word::WordStudy> {
        let index = self
            .lemma_index
            .get_or_init(|| LemmaIndex::build(&self.verses, WitnessId::Lxx));
        index.word_study(&self.verses, query)
    }

    /// `scribe occurrences <query> [--book]`.
    pub fn lemma_occurrences(
        &self,
        query: &str,
        book: Option<BookId>,
    ) -> Result<crate::domain::word::OccurrenceList> {
        let index = self
            .lemma_index
            .get_or_init(|| LemmaIndex::build(&self.verses, WitnessId::Lxx));
        index.occurrences(&self.verses, query, book)
    }

    fn witness_installed(&self, witness: WitnessId) -> bool {
        self.data_dir
            .join("store")
            .join(witness.store_file())
            .exists()
    }

    fn verse_slice(
        &self,
        witness: WitnessId,
        book: BookId,
        chapter: u16,
    ) -> Option<&[ScriptureText]> {
        let range = self.by_ref.get(&(witness, book, chapter))?;
        Some(&self.verses[range.start as usize..range.end as usize])
    }

    fn max_verse(&self, witness: WitnessId, book: BookId, chapter: u16) -> Option<u16> {
        self.verse_slice(witness, book, chapter)
            .and_then(|vs| vs.last().map(|v| v.verse.get()))
    }

    fn max_chapter(&self, witness: WitnessId, book: BookId) -> Option<u16> {
        let mut max = None;
        for (w, b, ch) in self.by_ref.keys() {
            if *w == witness && *b == book {
                max = Some(max.map_or(*ch, |m: u16| m.max(*ch)));
            }
        }
        max
    }

    fn validate(&self, reference: &PassageRef, witness: WitnessId) -> Result<()> {
        let max_chapter = self.max_chapter(witness, reference.book).ok_or_else(|| {
            ScribeError::BookNotInWitness {
                book: reference.book.canonical_name().to_string(),
                witness: witness.meta().title.to_string(),
            }
        })?;
        if reference.chapter.get() > max_chapter {
            return Err(ScribeError::ChapterOutOfRange {
                book: reference.book.canonical_name().to_string(),
                chapter: reference.chapter.get(),
                max: max_chapter,
            });
        }
        let max_verse = self
            .max_verse(witness, reference.book, reference.chapter.get())
            .unwrap_or(0);
        if reference.start_verse.get() > max_verse {
            return Err(ScribeError::VerseOutOfRange {
                book: reference.book.canonical_name().to_string(),
                chapter: reference.chapter.get(),
                verse: reference.start_verse.get(),
                max: max_verse,
            });
        }
        Ok(())
    }
}

impl ScriptureSource for Store {
    fn passage(&self, reference: &PassageRef, witness: WitnessId) -> Result<Passage> {
        if !self.witness_installed(witness) {
            return match witness {
                WitnessId::KjvApocrypha => Err(ScribeError::KjvaNotInstalled),
                WitnessId::Lxx => Err(ScribeError::GreekNotInstalled {
                    book: reference.book.canonical_name().to_string(),
                }),
            };
        }
        self.validate(reference, witness)?;
        let slice = self
            .verse_slice(witness, reference.book, reference.chapter.get())
            .ok_or_else(|| ScribeError::BookNotInWitness {
                book: reference.book.canonical_name().to_string(),
                witness: witness.meta().title.to_string(),
            })?;
        let verses: Vec<ScriptureText> = slice
            .iter()
            .filter(|v| {
                v.verse.get() >= reference.start_verse.get()
                    && (reference.is_chapter() || v.verse.get() <= reference.end_verse.get())
            })
            .cloned()
            .collect();
        if verses.is_empty() {
            return Err(ScribeError::VerseOutOfRange {
                book: reference.book.canonical_name().to_string(),
                chapter: reference.chapter.get(),
                verse: reference.start_verse.get(),
                max: self
                    .max_verse(witness, reference.book, reference.chapter.get())
                    .unwrap_or(0),
            });
        }
        Ok(Passage {
            reference: *reference,
            witness,
            verses,
        })
    }

    fn chapter(&self, book: BookId, chapter: ChapterNumber, witness: WitnessId) -> Result<Chapter> {
        let passage = self.passage(&PassageRef::chapter(book, chapter), witness)?;
        Ok(Chapter {
            book,
            chapter,
            witness,
            verses: passage.verses,
        })
    }

    fn search(&self, query: &SearchQuery) -> Result<Vec<SearchHit>> {
        if !self.witness_installed(query.witness) {
            return match query.witness {
                WitnessId::KjvApocrypha => Err(ScribeError::KjvaNotInstalled),
                WitnessId::Lxx => Err(ScribeError::GreekNotInstalled {
                    book: "the corpus".to_string(),
                }),
            };
        }
        if query.terms.is_empty() {
            return Ok(Vec::new());
        }
        // Intersect postings of all terms.
        let mut candidate: Option<Vec<u32>> = None;
        for term in &query.terms {
            let postings = self.index.get(term);
            match (&mut candidate, postings) {
                (None, None) => return Ok(Vec::new()),
                (None, Some(p)) => candidate = Some(p.clone()),
                (Some(c), None) => {
                    c.clear();
                    return Ok(Vec::new());
                }
                (Some(c), Some(p)) => {
                    let mut out = Vec::new();
                    let (mut i, mut j) = (0usize, 0usize);
                    while i < c.len() && j < p.len() {
                        match c[i].cmp(&p[j]) {
                            std::cmp::Ordering::Less => i += 1,
                            std::cmp::Ordering::Greater => j += 1,
                            std::cmp::Ordering::Equal => {
                                out.push(c[i]);
                                i += 1;
                                j += 1;
                            }
                        }
                    }
                    *c = out;
                }
            }
        }
        let mut hits: Vec<SearchHit> = Vec::new();
        if let Some(cands) = candidate {
            for idx in cands {
                let verse = &self.verses[idx as usize];
                if verse.witness != query.witness {
                    continue;
                }
                if let Some(book) = query.book {
                    if verse.book != book {
                        continue;
                    }
                }
                let mut score = 0u32;
                for term in &query.terms {
                    score += verse
                        .tokens
                        .iter()
                        .filter(|t| &t.normalized == term)
                        .count() as u32;
                }
                if score > 0 {
                    hits.push(SearchHit {
                        witness: verse.witness,
                        book: verse.book,
                        chapter: verse.chapter,
                        verse: verse.verse,
                        text: verse.text.clone(),
                        score,
                    });
                }
            }
        }
        hits.sort_by(|a, b| {
            b.score.cmp(&a.score).then_with(|| {
                a.book
                    .canonical_name()
                    .cmp(b.book.canonical_name())
                    .then(a.chapter.cmp(&b.chapter))
                    .then(a.verse.cmp(&b.verse))
            })
        });
        hits.truncate(query.limit);
        Ok(hits)
    }

    fn book_info(&self, book: BookId, witness: WitnessId) -> Result<BookInfo> {
        if !self.witness_installed(witness) {
            return match witness {
                WitnessId::KjvApocrypha => Err(ScribeError::KjvaNotInstalled),
                WitnessId::Lxx => Err(ScribeError::GreekNotInstalled {
                    book: book.canonical_name().to_string(),
                }),
            };
        }
        let mut chapters: Vec<(ChapterNumber, u16)> = Vec::new();
        let mut keys: Vec<u16> = self
            .by_ref
            .keys()
            .filter(|(w, b, _)| *w == witness && *b == book)
            .map(|(_, _, c)| *c)
            .collect();
        keys.sort_unstable();
        for ch in keys {
            let n = self.max_verse(witness, book, ch).unwrap_or(0);
            chapters.push((ChapterNumber::new(ch), n));
        }
        Ok(BookInfo {
            book,
            witness,
            chapters,
        })
    }

    fn datasets(&self) -> Vec<DatasetInfo> {
        WitnessId::ALL
            .iter()
            .map(|w| {
                let path = self.data_dir.join("store").join(w.store_file());
                DatasetInfo {
                    witness: w.meta(),
                    available: path.exists(),
                    verses: self.count_verses(*w),
                    path: if path.exists() {
                        Some(path.display().to_string())
                    } else {
                        None
                    },
                }
            })
            .collect()
    }
}

/// Read the provenance record written by the importers.
pub fn read_provenance(data_dir: &Path, witness: WitnessId) -> Option<Provenance> {
    let path = data_dir
        .join("meta")
        .join(format!("{}.json", witness.dataset_name()));
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}
