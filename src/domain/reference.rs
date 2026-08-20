//! Strong types for Scripture references and the reference parser.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use super::book::{resolve_book, BookId};

/// Chapter number (1-based; 0 is reserved for material such as the Greek
/// Sirach prologue that precedes chapter 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ChapterNumber(pub u16);

/// Verse number (1-based).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct VerseNumber(pub u16);

impl ChapterNumber {
    pub fn new(n: u16) -> Self {
        ChapterNumber(n)
    }
    pub fn get(self) -> u16 {
        self.0
    }
}

impl VerseNumber {
    pub fn new(n: u16) -> Self {
        VerseNumber(n)
    }
    pub fn get(self) -> u16 {
        self.0
    }
}

/// A parsed passage reference: one book, one chapter, and an inclusive verse
/// range (a chapter reference uses the full chapter).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PassageRef {
    pub book: BookId,
    pub chapter: ChapterNumber,
    /// First verse (1 for whole-chapter references).
    pub start_verse: VerseNumber,
    /// Last verse, inclusive (max u16 = "to end of chapter").
    pub end_verse: VerseNumber,
}

impl PassageRef {
    /// A whole-chapter reference.
    pub fn chapter(book: BookId, chapter: ChapterNumber) -> Self {
        PassageRef {
            book,
            chapter,
            start_verse: VerseNumber::new(1),
            end_verse: VerseNumber::new(u16::MAX),
        }
    }

    /// A single-verse reference.
    pub fn verse(book: BookId, chapter: ChapterNumber, verse: VerseNumber) -> Self {
        PassageRef {
            book,
            chapter,
            start_verse: verse,
            end_verse: verse,
        }
    }

    pub fn is_chapter(&self) -> bool {
        self.end_verse.0 == u16::MAX
    }

    pub fn is_single_verse(&self) -> bool {
        self.start_verse == self.end_verse
    }

    /// Human form: `Sirach 2:1` or `Sirach 2`.
    pub fn to_display(self) -> String {
        if self.is_chapter() {
            format!("{} {}", self.book.canonical_name(), self.chapter.0)
        } else if self.is_single_verse() {
            format!(
                "{} {}:{}",
                self.book.canonical_name(),
                self.chapter.0,
                self.start_verse.0
            )
        } else {
            format!(
                "{} {}:{}-{}",
                self.book.canonical_name(),
                self.chapter.0,
                self.start_verse.0,
                self.end_verse.0
            )
        }
    }

    /// Short form used in search hit headers: `Sirach 2:1`.
    pub fn to_short(self) -> String {
        if self.is_chapter() {
            format!("{} {}", self.book.canonical_name(), self.chapter.0)
        } else {
            format!(
                "{} {}:{}",
                self.book.canonical_name(),
                self.chapter.0,
                self.start_verse.0
            )
        }
    }
}

impl fmt::Display for PassageRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_display())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ReferenceParseError {
    #[error("missing reference after book name (examples: \"sirach 2\", \"sirach 2:1\", \"wisdom 2:12-20\")")]
    MissingReference,
    #[error("malformed reference {raw:?}: expected N, N:M or N:M-P (examples: \"sirach 2\", \"sirach 2:1\", \"sirach 2:1-5\")")]
    Malformed { raw: String },
    #[error("verse range end {end} is before start {start} in {raw:?}")]
    ReversedRange { start: u16, end: u16, raw: String },
    #[error("chapter number out of range in {raw:?}")]
    ChapterTooLarge { raw: String },
    #[error("verse number out of range in {raw:?}")]
    VerseTooLarge { raw: String },
    #[error(transparent)]
    Book(#[from] super::book::BookLookupError),
}

/// Parse a free-form reference string such as:
/// `sirach 2`, `Sirach 2:1`, `wisdom 2:12-20`, `1 maccabees 3:1`,
/// `Ecclesiasticus 2:1`.
pub fn parse_reference(input: &str) -> Result<PassageRef, ReferenceParseError> {
    let (book, alias_len) = resolve_book(input)?;
    let rest = input.trim()[char_len(input.trim(), alias_len)..].trim();
    if rest.is_empty() {
        return Err(ReferenceParseError::MissingReference);
    }
    let rest = rest.to_lowercase().replace(' ', "");
    parse_ref_suffix(book, &rest)
}

fn char_len(s: &str, chars: usize) -> usize {
    s.char_indices()
        .nth(chars)
        .map(|(i, _)| i)
        .unwrap_or(s.len())
}

fn parse_ref_suffix(book: BookId, rest: &str) -> Result<PassageRef, ReferenceParseError> {
    let err = |raw: String| ReferenceParseError::Malformed { raw };
    let parts: Vec<&str> = rest.split(':').collect();
    match parts.len() {
        1 => {
            // chapter only
            let ch = parse_u16(parts[0]).ok_or_else(|| err(rest.to_string()))?;
            if ch > 200 {
                return Err(ReferenceParseError::ChapterTooLarge {
                    raw: rest.to_string(),
                });
            }
            Ok(PassageRef::chapter(book, ChapterNumber::new(ch)))
        }
        2 => {
            let ch = parse_u16(parts[0]).ok_or_else(|| err(rest.to_string()))?;
            if ch > 200 {
                return Err(ReferenceParseError::ChapterTooLarge {
                    raw: rest.to_string(),
                });
            }
            let v = parts[1];
            if let Some((s, e)) = v.split_once('-') {
                let s = parse_u16(s).ok_or_else(|| err(rest.to_string()))?;
                let e = parse_u16(e).ok_or_else(|| err(rest.to_string()))?;
                if e < s {
                    return Err(ReferenceParseError::ReversedRange {
                        start: s,
                        end: e,
                        raw: rest.to_string(),
                    });
                }
                if e > 300 {
                    return Err(ReferenceParseError::VerseTooLarge {
                        raw: rest.to_string(),
                    });
                }
                Ok(PassageRef {
                    book,
                    chapter: ChapterNumber::new(ch),
                    start_verse: VerseNumber::new(s),
                    end_verse: VerseNumber::new(e),
                })
            } else {
                let s = parse_u16(v).ok_or_else(|| err(rest.to_string()))?;
                if s > 300 {
                    return Err(ReferenceParseError::VerseTooLarge {
                        raw: rest.to_string(),
                    });
                }
                Ok(PassageRef::verse(
                    book,
                    ChapterNumber::new(ch),
                    VerseNumber::new(s),
                ))
            }
        }
        _ => Err(err(rest.to_string())),
    }
}

fn parse_u16(s: &str) -> Option<u16> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    s.parse().ok()
}

impl FromStr for PassageRef {
    type Err = ReferenceParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_reference(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(n: u16) -> VerseNumber {
        VerseNumber::new(n)
    }

    #[test]
    fn parses_chapter() {
        let r = parse_reference("Sirach 2").unwrap();
        assert_eq!(r.book, BookId::Sirach);
        assert_eq!(r.chapter, ChapterNumber::new(2));
        assert!(r.is_chapter());
    }

    #[test]
    fn parses_verse() {
        let r = parse_reference("Sirach 2:1").unwrap();
        assert_eq!(r.book, BookId::Sirach);
        assert_eq!(r.chapter, ChapterNumber::new(2));
        assert_eq!(r.start_verse, v(1));
        assert_eq!(r.end_verse, v(1));
    }

    #[test]
    fn parses_range() {
        let r = parse_reference("Sirach 2:1-5").unwrap();
        assert_eq!(r.start_verse, v(1));
        assert_eq!(r.end_verse, v(5));
        assert!(!r.is_single_verse());
    }

    #[test]
    fn parses_ecclesiasticus_alias() {
        let r = parse_reference("Ecclesiasticus 2:1").unwrap();
        assert_eq!(r.book, BookId::Sirach);
        assert_eq!(r.chapter, ChapterNumber::new(2));
    }

    #[test]
    fn parses_numbered_book_with_separate_tokens() {
        let r = parse_reference("1 Maccabees 3:1").unwrap();
        assert_eq!(r.book, BookId::FirstMaccabees);
        assert_eq!(r.chapter, ChapterNumber::new(3));
        assert_eq!(r.start_verse, v(1));
    }

    #[test]
    fn parses_wisdom_range() {
        let r = parse_reference("Wisdom 2:12-20").unwrap();
        assert_eq!(r.book, BookId::WisdomOfSolomon);
        assert_eq!(r.start_verse, v(12));
        assert_eq!(r.end_verse, v(20));
    }

    #[test]
    fn rejects_reversed_range() {
        assert!(matches!(
            parse_reference("Sirach 2:5-1"),
            Err(ReferenceParseError::ReversedRange { .. })
        ));
    }

    #[test]
    fn rejects_garbage() {
        assert!(matches!(
            parse_reference("Sirach x"),
            Err(ReferenceParseError::Malformed { .. })
        ));
        assert!(matches!(
            parse_reference("Sirach 2:1:3"),
            Err(ReferenceParseError::Malformed { .. })
        ));
        assert!(matches!(
            parse_reference("Sirach"),
            Err(ReferenceParseError::MissingReference)
        ));
        assert!(matches!(
            parse_reference("Sirach 2:1-"),
            Err(ReferenceParseError::Malformed { .. })
        ));
    }

    #[test]
    fn display_forms() {
        assert_eq!(
            parse_reference("Sirach 2").unwrap().to_display(),
            "Sirach 2"
        );
        assert_eq!(
            parse_reference("Sirach 2:1").unwrap().to_display(),
            "Sirach 2:1"
        );
        assert_eq!(
            parse_reference("Wisdom 2:12-20").unwrap().to_display(),
            "Wisdom of Solomon 2:12-20"
        );
    }
}
