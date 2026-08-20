//! The lemma index: normalized surface → lemma(s), and normalized lemma →
//! token occurrences. Built once at store load from the Greek witness tokens;
//! for an Apocrypha-sized corpus (~70k Greek tokens) this is a few
//! milliseconds in release mode (measured, see README).

use std::collections::HashMap;

use unicode_normalization::UnicodeNormalization;

use crate::domain::book::BookId;
use crate::domain::passage::ScriptureText;
use crate::domain::reference::PassageRef;
use crate::domain::witness::WitnessId;
use crate::domain::word::{
    occurrence_from_token, BookCount, Corpus, LemmaCandidate, LemmaResolution, LemmaStudy,
    Morphology, Occurrence, OccurrenceList, PartOfSpeech, WordForm, WordStudy,
};
use crate::error::{Result, ScribeError};
use crate::greek::transliterate;
use crate::text::normalize;

/// One token occurrence of a lemma: verse index + position within the verse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LemmaOccurrence {
    pub verse_idx: u32,
    pub token_pos: u16,
}

/// Compact in-memory index over the Greek corpus.
#[derive(Debug, Default)]
pub struct LemmaIndex {
    /// normalized lemma → occurrences.
    lemma_occurrences: HashMap<String, Vec<LemmaOccurrence>>,
    /// normalized lemma → display lemma (first seen in the corpus).
    lemma_display: HashMap<String, String>,
    /// normalized surface → distinct normalized lemmas (first-seen order).
    surface_lemmas: HashMap<String, Vec<String>>,
}

impl LemmaIndex {
    pub fn build(verses: &[ScriptureText], witness: WitnessId) -> LemmaIndex {
        let mut index = LemmaIndex::default();
        // Normalizing a lemma is the expensive part of building the index;
        // distinct lemmas number in the thousands, so memoize per lemma.
        let mut norm_memo: HashMap<String, String> = HashMap::new();
        for (verse_idx, verse) in verses.iter().enumerate() {
            if verse.witness != witness {
                continue;
            }
            for (token_pos, token) in verse.tokens.iter().enumerate() {
                let Some(lemma) = token.lemma.as_deref() else {
                    continue;
                };
                let norm_lemma = match norm_memo.get(lemma) {
                    Some(nl) => nl.clone(),
                    None => {
                        let nl = normalize(lemma);
                        if nl.is_empty() {
                            continue;
                        }
                        norm_memo.insert(lemma.to_string(), nl.clone());
                        nl
                    }
                };
                index
                    .lemma_occurrences
                    .entry(norm_lemma.clone())
                    .or_default()
                    .push(LemmaOccurrence {
                        verse_idx: verse_idx as u32,
                        token_pos: token_pos as u16,
                    });
                index
                    .lemma_display
                    .entry(norm_lemma.clone())
                    .or_insert_with(|| lemma.to_string());
                let lemmas = index
                    .surface_lemmas
                    .entry(token.normalized.clone())
                    .or_default();
                if !lemmas.contains(&norm_lemma) {
                    lemmas.push(norm_lemma);
                }
            }
        }
        index
    }

    pub fn is_empty(&self) -> bool {
        self.lemma_occurrences.is_empty()
    }

    fn lemma_occurrences(&self, norm_lemma: &str) -> &[LemmaOccurrence] {
        self.lemma_occurrences
            .get(norm_lemma)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    fn display_lemma(&self, norm_lemma: &str) -> Option<&str> {
        self.lemma_display.get(norm_lemma).map(String::as_str)
    }

    /// Resolve a user query (dictionary form or inflected surface) to a lemma
    /// study. The candidate set is the union of:
    /// * the lemma whose dictionary form matches the query, and
    /// * every lemma whose tokens include the query as a surface form.
    ///
    /// Zero candidates → `LemmaNotFound`; more than one → `AmbiguousLemma`
    /// (no silent guessing).
    pub fn word_study(&self, verses: &[ScriptureText], query: &str) -> Result<WordStudy> {
        if self.is_empty() {
            return Err(ScribeError::GreekDataNotInstalled);
        }
        let normalized_query = normalize(query);
        let candidates = self.resolve_candidates(query, &normalized_query)?;
        let resolution = match candidates.len() {
            0 => {
                return Err(ScribeError::LemmaNotFound {
                    query: query.to_string(),
                })
            }
            1 => self.study(verses, &candidates[0]),
            _ => {
                let candidates: Vec<LemmaCandidate> = candidates
                    .iter()
                    .map(|nl| self.candidate(verses, nl))
                    .collect();
                return Err(ScribeError::AmbiguousLemma {
                    surface: query.to_string(),
                    list: candidate_list(&candidates),
                });
            }
        };
        Ok(WordStudy {
            query: query.to_string(),
            normalized_query,
            corpus: Corpus::from_witness(WitnessId::Lxx),
            resolution,
        })
    }

    /// Resolve a query to candidate normalized lemmas, in stable order.
    ///
    /// 1. An exact dictionary-form match (NFC, case-insensitive,
    ///    accent-sensitive) wins outright: the user typed the lemma.
    /// 2. Otherwise the union of the lemma-key match and every lemma whose
    ///    tokens include the query as a (normalized) surface form.
    fn resolve_candidates(&self, query: &str, normalized_query: &str) -> Result<Vec<String>> {
        if normalized_query.is_empty() {
            return Err(ScribeError::Other(format!(
                "word query {query:?} contains no searchable Greek letters"
            )));
        }
        // Exact dictionary-form match.
        let nfc: String = query.nfkc().collect();
        let lower = nfc.to_lowercase();
        if let Some((nl, _)) = self
            .lemma_display
            .iter()
            .find(|(_, d)| d.to_lowercase() == lower)
        {
            return Ok(vec![nl.clone()]);
        }
        let mut candidates: Vec<String> = Vec::new();
        if self.lemma_occurrences.contains_key(normalized_query) {
            candidates.push(normalized_query.to_string());
        }
        if let Some(lemmas) = self.surface_lemmas.get(normalized_query) {
            for l in lemmas {
                if !candidates.contains(l) {
                    candidates.push(l.clone());
                }
            }
        }
        Ok(candidates)
    }

    fn study(&self, verses: &[ScriptureText], norm_lemma: &str) -> LemmaResolution {
        let occs = self.lemma_occurrences(norm_lemma);
        let display = self
            .display_lemma(norm_lemma)
            .unwrap_or(norm_lemma)
            .to_string();

        let mut forms: Vec<WordForm> = Vec::new();
        let mut pos_votes: Vec<PartOfSpeech> = Vec::new();
        let mut books: Vec<BookCount> = Vec::new();
        for occ in occs {
            let verse = &verses[occ.verse_idx as usize];
            let token = &verse.tokens[occ.token_pos as usize];
            if let Some(f) = forms.iter_mut().find(|f| f.normalized == token.normalized) {
                f.count += 1;
            } else {
                forms.push(WordForm {
                    surface: token.surface.clone(),
                    normalized: token.normalized.clone(),
                    morphology: token.morph.clone(),
                    count: 1,
                });
            }
            if let Some(morph) = token.morph.as_deref() {
                if let Some(pos) = Morphology(morph.to_string()).part_of_speech() {
                    pos_votes.push(pos);
                }
            }
            if let Some(bc) = books.iter_mut().find(|b| b.book == verse.book) {
                bc.count += 1;
            } else {
                books.push(BookCount {
                    book: verse.book,
                    count: 1,
                });
            }
        }
        forms.sort_by(|a, b| b.count.cmp(&a.count).then(a.surface.cmp(&b.surface)));
        books.sort_by_key(|b| {
            BookId::ALL
                .iter()
                .position(|x| *x == b.book)
                .unwrap_or(usize::MAX)
        });
        let transliteration = Some(transliterate(&display));
        LemmaResolution::Found(Box::new(LemmaStudy {
            lemma: display,
            normalized_lemma: norm_lemma.to_string(),
            transliteration,
            part_of_speech: most_common_pos(&pos_votes),
            total_occurrences: occs.len() as u32,
            forms,
            books,
        }))
    }

    fn candidate(&self, verses: &[ScriptureText], norm_lemma: &str) -> LemmaCandidate {
        let mut pos_votes = Vec::new();
        for occ in self.lemma_occurrences(norm_lemma) {
            let verse = &verses[occ.verse_idx as usize];
            let token = &verse.tokens[occ.token_pos as usize];
            if let Some(morph) = token.morph.as_deref() {
                if let Some(pos) = Morphology(morph.to_string()).part_of_speech() {
                    pos_votes.push(pos);
                }
            }
        }
        LemmaCandidate {
            lemma: self
                .display_lemma(norm_lemma)
                .unwrap_or(norm_lemma)
                .to_string(),
            normalized_lemma: norm_lemma.to_string(),
            part_of_speech: most_common_pos(&pos_votes),
            occurrence_count: self.lemma_occurrences(norm_lemma).len() as u32,
        }
    }

    /// All occurrences of a resolved lemma, optionally restricted to a book.
    pub fn occurrences(
        &self,
        verses: &[ScriptureText],
        query: &str,
        book: Option<BookId>,
    ) -> Result<OccurrenceList> {
        if self.is_empty() {
            return Err(ScribeError::GreekDataNotInstalled);
        }
        let normalized_query = normalize(query);
        let candidates = self.resolve_candidates(query, &normalized_query)?;
        let norm_lemma = match candidates.len() {
            0 => {
                return Err(ScribeError::LemmaNotFound {
                    query: query.to_string(),
                })
            }
            1 => candidates[0].clone(),
            _ => {
                let candidates: Vec<LemmaCandidate> = candidates
                    .iter()
                    .map(|nl| self.candidate(verses, nl))
                    .collect();
                return Err(ScribeError::AmbiguousLemma {
                    surface: query.to_string(),
                    list: candidate_list(&candidates),
                });
            }
        };

        let mut occs: Vec<Occurrence> = Vec::new();
        for occ in self.lemma_occurrences(&norm_lemma) {
            let verse = &verses[occ.verse_idx as usize];
            if let Some(book) = book {
                if verse.book != book {
                    continue;
                }
            }
            let token = &verse.tokens[occ.token_pos as usize];
            occs.push(occurrence_from_token(
                PassageRef::verse(verse.book, verse.chapter, verse.verse),
                verse.witness,
                &verse.text,
                occ.token_pos as usize,
                token,
                self.display_lemma(&norm_lemma).unwrap_or(&norm_lemma),
            ));
        }
        occs.sort_by(|a, b| {
            let abook = BookId::ALL
                .iter()
                .position(|x| *x == a.reference.book)
                .unwrap_or(usize::MAX);
            let bbook = BookId::ALL
                .iter()
                .position(|x| *x == b.reference.book)
                .unwrap_or(usize::MAX);
            abook
                .cmp(&bbook)
                .then(a.reference.chapter.cmp(&b.reference.chapter))
                .then(a.reference.start_verse.cmp(&b.reference.start_verse))
                .then(a.position.cmp(&b.position))
        });
        Ok(OccurrenceList {
            query: query.to_string(),
            normalized_query,
            lemma: self
                .display_lemma(&norm_lemma)
                .unwrap_or(&norm_lemma)
                .to_string(),
            normalized_lemma: norm_lemma.clone(),
            corpus: Corpus::from_witness(WitnessId::Lxx),
            book_filter: book,
            total: occs.len() as u32,
            occurrences: occs,
        })
    }
}

fn most_common_pos(votes: &[PartOfSpeech]) -> Option<PartOfSpeech> {
    let mut counts: HashMap<PartOfSpeech, u32> = HashMap::new();
    for v in votes {
        *counts.entry(*v).or_insert(0) += 1;
    }
    counts
        .into_iter()
        .max_by_key(|(pos, n)| (*n, -(*pos as i32)))
        .map(|(pos, _)| pos)
}

fn candidate_list(candidates: &[LemmaCandidate]) -> String {
    candidates
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let pos = c
                .part_of_speech
                .map(|p| p.label().to_string())
                .unwrap_or_else(|| "?".to_string());
            format!(
                "{}. {} ({}, {} occurrences)",
                i + 1,
                c.lemma,
                pos,
                c.occurrence_count
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::book::BookId;
    use crate::domain::passage::{ScriptureText, Token};
    use crate::domain::reference::{ChapterNumber, VerseNumber};
    use crate::error::ScribeError;

    fn tok(surface: &str, morph: &str, lemma: &str) -> Token {
        Token {
            surface: surface.to_string(),
            normalized: normalize(surface),
            lemma: Some(lemma.to_string()),
            morph: Some(morph.to_string()),
        }
    }

    fn verse(book: BookId, ch: u16, v: u16, tokens: Vec<Token>) -> ScriptureText {
        ScriptureText {
            witness: WitnessId::Lxx,
            book,
            chapter: ChapterNumber::new(ch),
            verse: VerseNumber::new(v),
            text: tokens
                .iter()
                .map(|t| t.surface.clone())
                .collect::<Vec<_>>()
                .join(" "),
            tokens,
            source_reference: None,
        }
    }

    fn peirasmos_corpus() -> Vec<ScriptureText> {
        vec![
            verse(
                BookId::Sirach,
                2,
                1,
                vec![
                    tok("τέκνον", "N2N-VSN---", "τέκνον"),
                    tok("πειρασμόν", "N2--ASM---", "πειρασμός"),
                ],
            ),
            verse(
                BookId::Sirach,
                27,
                5,
                vec![tok("πειρασμὸς", "N2--NSM---", "πειρασμός")],
            ),
            verse(
                BookId::FirstMaccabees,
                2,
                52,
                vec![tok("πειρασμῷ", "N2--DSM---", "πειρασμός")],
            ),
        ]
    }

    #[test]
    fn surface_form_resolves_to_lemma() {
        let verses = peirasmos_corpus();
        let index = LemmaIndex::build(&verses, WitnessId::Lxx);
        let study = index.word_study(&verses, "πειρασμόν").unwrap();
        let LemmaResolution::Found(found) = &study.resolution else {
            panic!("expected Found");
        };
        assert_eq!(found.lemma, "πειρασμός");
        assert_eq!(found.total_occurrences, 3);
        assert_eq!(found.forms.len(), 3);
        assert_eq!(found.part_of_speech, Some(PartOfSpeech::Noun));
    }

    #[test]
    fn uppercase_and_accentless_queries_equivalent() {
        let verses = peirasmos_corpus();
        let index = LemmaIndex::build(&verses, WitnessId::Lxx);
        for q in ["πειρασμός", "ΠΕΙΡΑΣΜΟΣ", "πειρασμος"] {
            let study = index.word_study(&verses, q).unwrap();
            let LemmaResolution::Found(found) = &study.resolution else {
                panic!("expected Found for {q}");
            };
            assert_eq!(found.lemma, "πειρασμός");
        }
    }

    #[test]
    fn ambiguous_surface_lists_candidates() {
        let verses = vec![
            verse(
                BookId::Sirach,
                1,
                1,
                vec![tok("ἀλλά", "C---------", "ἀλλά")],
            ),
            verse(
                BookId::Sirach,
                2,
                1,
                vec![tok("ἄλλα", "A1--APN---", "ἄλλος")],
            ),
        ];
        let index = LemmaIndex::build(&verses, WitnessId::Lxx);
        let err = index.word_study(&verses, "ἄλλα").unwrap_err();
        match err {
            ScribeError::AmbiguousLemma { surface, list } => {
                assert_eq!(surface, "ἄλλα");
                assert!(list.contains("ἀλλά"));
                assert!(list.contains("ἄλλος"));
            }
            other => panic!("expected AmbiguousLemma, got {other}"),
        }
    }

    #[test]
    fn exact_dictionary_form_wins() {
        let verses = vec![
            verse(
                BookId::Sirach,
                1,
                1,
                vec![tok("ἀλλά", "C---------", "ἀλλά")],
            ),
            verse(
                BookId::Sirach,
                2,
                1,
                vec![tok("ἄλλα", "A1--APN---", "ἄλλος")],
            ),
        ];
        let index = LemmaIndex::build(&verses, WitnessId::Lxx);
        let study = index.word_study(&verses, "ἀλλά").unwrap();
        let LemmaResolution::Found(found) = &study.resolution else {
            panic!("expected Found");
        };
        assert_eq!(found.lemma, "ἀλλά");
    }

    #[test]
    fn lemma_occurrences_with_book_filter() {
        let verses = peirasmos_corpus();
        let index = LemmaIndex::build(&verses, WitnessId::Lxx);
        let all = index.occurrences(&verses, "πειρασμός", None).unwrap();
        assert_eq!(all.total, 3);
        assert!(all
            .occurrences
            .iter()
            .any(|o| o.reference.book == BookId::Sirach && o.reference.chapter.get() == 2));
        let sir = index
            .occurrences(&verses, "πειρασμόν", Some(BookId::Sirach))
            .unwrap();
        assert_eq!(sir.total, 2);
        assert!(sir
            .occurrences
            .iter()
            .all(|o| o.reference.book == BookId::Sirach));
        assert_eq!(sir.lemma, "πειρασμός");
    }

    #[test]
    fn not_found_is_an_error() {
        let verses = peirasmos_corpus();
        let index = LemmaIndex::build(&verses, WitnessId::Lxx);
        let err = index.word_study(&verses, "foobar").unwrap_err();
        assert!(matches!(err, ScribeError::LemmaNotFound { .. }));
    }

    #[test]
    fn empty_corpus_means_greek_not_installed() {
        let index = LemmaIndex::default();
        let err = index.word_study(&[], "πειρασμός").unwrap_err();
        assert!(matches!(err, ScribeError::GreekDataNotInstalled));
    }
}
