//! Compact binary cache of the store, written at import time and read on
//! every invocation. Keeps startup in the single-digit-millisecond range for
//! the Apocrypha corpus (~10k verses) while the JSONL remains the
//! human-inspectable source of truth and the fallback.

use std::io::{BufWriter, Write};
use std::path::Path;

use crate::domain::book::BookId;
use crate::domain::passage::{ScriptureText, Token};
use crate::domain::reference::{ChapterNumber, VerseNumber};
use crate::domain::witness::WitnessId;
use crate::error::{Result, ScribeError};

const MAGIC: &[u8; 8] = b"SCRIBEC2";

fn witness_tag(w: WitnessId) -> u32 {
    match w {
        WitnessId::KjvApocrypha => 0,
        WitnessId::Lxx => 1,
    }
}

fn book_index(book: BookId) -> Option<u16> {
    BookId::ALL
        .iter()
        .position(|b| *b == book)
        .map(|i| i as u16)
}

fn book_from_index(i: u16) -> Option<BookId> {
    BookId::ALL.get(i as usize).copied()
}

/// Serialize verses into the cache file at `path` (atomic: temp + rename).
pub fn write(path: &Path, witness: WitnessId, verses: &[ScriptureText]) -> Result<()> {
    let tmp = path.with_extension("cache.tmp");
    let file = std::fs::File::create(&tmp).map_err(|e| ScribeError::Io {
        path: tmp.display().to_string(),
        source: e,
    })?;
    let mut f = BufWriter::new(file);
    f.write_all(MAGIC).map_err(io_err(&tmp))?;
    f.write_all(&witness_tag(witness).to_le_bytes())
        .map_err(io_err(&tmp))?;
    f.write_all(&(verses.len() as u32).to_le_bytes())
        .map_err(io_err(&tmp))?;
    for v in verses {
        let book = book_index(v.book).ok_or_else(|| ScribeError::ImportFailed {
            dataset: witness.dataset_name().into(),
            detail: format!("book {} outside canon", v.book.canonical_name()),
        })?;
        f.write_all(&book.to_le_bytes()).map_err(io_err(&tmp))?;
        f.write_all(&v.chapter.get().to_le_bytes())
            .map_err(io_err(&tmp))?;
        f.write_all(&v.verse.get().to_le_bytes())
            .map_err(io_err(&tmp))?;
        write_str(&mut f, &v.text, &tmp)?;
        write_opt_str(&mut f, v.source_reference.as_deref(), &tmp)?;
        f.write_all(&(v.tokens.len() as u32).to_le_bytes())
            .map_err(io_err(&tmp))?;
        for t in &v.tokens {
            write_str(&mut f, &t.surface, &tmp)?;
            write_str(&mut f, &t.normalized, &tmp)?;
            write_opt_str(&mut f, t.lemma.as_deref(), &tmp)?;
            write_opt_str(&mut f, t.morph.as_deref(), &tmp)?;
        }
    }
    f.flush().map_err(io_err(&tmp))?;
    drop(f);
    std::fs::rename(&tmp, path).map_err(|e| ScribeError::Io {
        path: path.display().to_string(),
        source: e,
    })?;
    Ok(())
}

fn write_str<W: Write>(f: &mut W, s: &str, tmp: &Path) -> Result<()> {
    f.write_all(&(s.len() as u32).to_le_bytes())
        .map_err(io_err(tmp))?;
    f.write_all(s.as_bytes()).map_err(io_err(tmp))?;
    Ok(())
}

fn write_opt_str<W: Write>(f: &mut W, s: Option<&str>, tmp: &Path) -> Result<()> {
    match s {
        Some(s) => {
            f.write_all(&1u8.to_le_bytes()).map_err(io_err(tmp))?;
            write_str(f, s, tmp)?;
        }
        None => {
            f.write_all(&0u8.to_le_bytes()).map_err(io_err(tmp))?;
        }
    }
    Ok(())
}

fn io_err(path: &Path) -> impl Fn(std::io::Error) -> ScribeError + '_ {
    move |e| ScribeError::Io {
        path: path.display().to_string(),
        source: e,
    }
}

/// Read the cache at `path`; returns `Ok(None)` when the file is missing or
/// does not match the expected format/version (caller falls back to JSONL).
pub fn read(path: &Path, witness: WitnessId) -> Result<Option<Vec<ScriptureText>>> {
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(_) => return Ok(None),
    };
    let mut cur = Cursor {
        bytes: &bytes,
        pos: 0,
    };
    let read_result = (|| -> Result<Vec<ScriptureText>> {
        if cur.take(8)? != &MAGIC[..] {
            return Ok(Vec::new()); // wrong magic → treat as absent
        }
        if cur.take_u32()? != witness_tag(witness) {
            return Ok(Vec::new());
        }
        let count = cur.take_u32()? as usize;
        let mut verses = Vec::with_capacity(count);
        for _ in 0..count {
            let book =
                book_from_index(cur.take_u16()?).ok_or_else(|| ScribeError::StoreCorrupt {
                    path: path.display().to_string(),
                    detail: "bad book index".into(),
                })?;
            let chapter = ChapterNumber::new(cur.take_u16()?);
            let verse = VerseNumber::new(cur.take_u16()?);
            let text = cur.take_str()?;
            let source_reference = if cur.take_u8()? == 1 {
                Some(cur.take_str()?)
            } else {
                None
            };
            let n_tokens = cur.take_u32()? as usize;
            let mut tokens = Vec::with_capacity(n_tokens);
            for _ in 0..n_tokens {
                let surface = cur.take_str()?;
                let normalized = cur.take_str()?;
                let lemma = if cur.take_u8()? == 1 {
                    Some(cur.take_str()?)
                } else {
                    None
                };
                let morph = if cur.take_u8()? == 1 {
                    Some(cur.take_str()?)
                } else {
                    None
                };
                tokens.push(Token {
                    surface,
                    normalized,
                    lemma,
                    morph,
                });
            }
            verses.push(ScriptureText {
                witness,
                book,
                chapter,
                verse,
                text,
                tokens,
                source_reference,
            });
        }
        Ok(verses)
    })();
    match read_result {
        Ok(v) if !v.is_empty() => Ok(Some(v)),
        Ok(_) => Ok(None),
        Err(_) => {
            // Corrupt cache: ignore and rebuild from JSONL.
            let _ = std::fs::remove_file(path);
            Ok(None)
        }
    }
}

struct Cursor<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        if self.pos + n > self.bytes.len() {
            return Err(ScribeError::StoreCorrupt {
                path: "cache".into(),
                detail: "truncated".into(),
            });
        }
        let s = &self.bytes[self.pos..self.pos + n];
        self.pos += n;
        Ok(s)
    }

    fn take_u32(&mut self) -> Result<u32> {
        let b: [u8; 4] = self.take(4)?.try_into().unwrap();
        Ok(u32::from_le_bytes(b))
    }

    fn take_u16(&mut self) -> Result<u16> {
        let b: [u8; 2] = self.take(2)?.try_into().unwrap();
        Ok(u16::from_le_bytes(b))
    }

    fn take_u8(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }

    fn take_str(&mut self) -> Result<String> {
        let len = self.take_u32()? as usize;
        let bytes = self.take(len)?;
        String::from_utf8(bytes.to_vec()).map_err(|_| ScribeError::StoreCorrupt {
            path: "cache".into(),
            detail: "invalid utf-8".into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::normalize;

    #[test]
    fn round_trips_verses() {
        let dir = std::env::temp_dir().join(format!("scribe-cache-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("t.cache");
        let verses = vec![
            ScriptureText {
                witness: WitnessId::KjvApocrypha,
                book: BookId::Sirach,
                chapter: ChapterNumber::new(2),
                verse: VerseNumber::new(1),
                text: "My son, if thou come to serve the Lord".to_string(),
                tokens: vec![Token {
                    surface: "Lord".into(),
                    normalized: normalize("Lord"),
                    lemma: None,
                    morph: None,
                }],
                source_reference: None,
            },
            ScriptureText {
                witness: WitnessId::KjvApocrypha,
                book: BookId::Sirach,
                chapter: ChapterNumber::new(2),
                verse: VerseNumber::new(2),
                text: "Set thy heart aright".to_string(),
                tokens: vec![],
                source_reference: None,
            },
        ];
        write(&path, WitnessId::KjvApocrypha, &verses).unwrap();
        let got = read(&path, WitnessId::KjvApocrypha).unwrap().unwrap();
        assert_eq!(got, verses);
        // wrong witness tag → None
        assert!(read(&path, WitnessId::Lxx).unwrap().is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
