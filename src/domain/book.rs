//! Canon of Apocrypha books and their names/aliases.
//!
//! The canon follows the CrossWire KJVA OSIS/module versification:
//! KJV 1769 Apocrypha, with the "Epistle of Jeremy" kept as its own book
//! (renumbered from Baruch 6 as in KJV printings).

use std::fmt;

use serde::{Deserialize, Serialize};

/// A stable identifier for an Apocrypha book.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BookId {
    FirstEsdras,
    SecondEsdras,
    Tobit,
    Judith,
    RestOfEsther,
    WisdomOfSolomon,
    Sirach,
    Baruch,
    EpistleOfJeremy,
    PrayerOfAzariah,
    Susanna,
    BelAndTheDragon,
    PrayerOfManasses,
    FirstMaccabees,
    SecondMaccabees,
}

impl BookId {
    pub const ALL: [BookId; 15] = [
        BookId::FirstEsdras,
        BookId::SecondEsdras,
        BookId::Tobit,
        BookId::Judith,
        BookId::RestOfEsther,
        BookId::WisdomOfSolomon,
        BookId::Sirach,
        BookId::Baruch,
        BookId::EpistleOfJeremy,
        BookId::PrayerOfAzariah,
        BookId::Susanna,
        BookId::BelAndTheDragon,
        BookId::PrayerOfManasses,
        BookId::FirstMaccabees,
        BookId::SecondMaccabees,
    ];

    /// The display name used in data files and output.
    pub fn canonical_name(self) -> &'static str {
        match self {
            BookId::FirstEsdras => "1 Esdras",
            BookId::SecondEsdras => "2 Esdras",
            BookId::Tobit => "Tobit",
            BookId::Judith => "Judith",
            BookId::RestOfEsther => "Rest of Esther",
            BookId::WisdomOfSolomon => "Wisdom of Solomon",
            BookId::Sirach => "Sirach",
            BookId::Baruch => "Baruch",
            BookId::EpistleOfJeremy => "Epistle of Jeremy",
            BookId::PrayerOfAzariah => "Prayer of Azariah",
            BookId::Susanna => "Susanna",
            BookId::BelAndTheDragon => "Bel and the Dragon",
            BookId::PrayerOfManasses => "Prayer of Manasses",
            BookId::FirstMaccabees => "1 Maccabees",
            BookId::SecondMaccabees => "2 Maccabees",
        }
    }

    /// The SWORD/LXXMorph-style short id used by the Greek corpus.
    pub fn greek_source_id(self) -> Option<&'static str> {
        match self {
            BookId::FirstEsdras => Some("1Esdr"),
            BookId::Tobit => Some("TobBA"),
            BookId::Judith => Some("Jdt"),
            BookId::WisdomOfSolomon => Some("Wis"),
            BookId::Sirach => Some("Sir"),
            BookId::Baruch => Some("Bar"),
            BookId::EpistleOfJeremy => Some("EpJer"),
            BookId::FirstMaccabees => Some("1Mac"),
            BookId::SecondMaccabees => Some("2Mac"),
            _ => None,
        }
    }

    /// All aliases (lowercase) that resolve to this book, longest first.
    pub fn aliases(self) -> &'static [&'static str] {
        match self {
            BookId::FirstEsdras => &["1 esdras", "i esdras", "first esdras", "1 esd", "esdras 1"],
            BookId::SecondEsdras => &[
                "2 esdras",
                "ii esdras",
                "second esdras",
                "2 esd",
                "esdras 2",
            ],
            BookId::Tobit => &["tobit", "tob", "tobias", "book of tobit"],
            BookId::Judith => &["judith", "jdt", "book of judith"],
            BookId::RestOfEsther => &[
                "rest of esther",
                "additions to esther",
                "additions of esther",
                "rest of the book of esther",
                "add esther",
            ],
            BookId::WisdomOfSolomon => &[
                "wisdom of solomon",
                "wisdom",
                "the wisdom of solomon",
                "wis",
            ],
            BookId::Sirach => &["sirach", "ecclesiasticus", "ecclus", "sir", "siracides"],
            BookId::Baruch => &["baruch", "bar", "book of baruch"],
            BookId::EpistleOfJeremy => &[
                "epistle of jeremy",
                "epistle of jeremiah",
                "epistle of jeremy the prophet",
                "epistle of jeremiah the prophet",
                "jeremy's epistle",
            ],
            BookId::PrayerOfAzariah => &[
                "prayer of azariah",
                "prayer of azarias",
                "song of the three children",
                "song of the three holy children",
                "azariah",
            ],
            BookId::Susanna => &[
                "susanna",
                "sus",
                "history of susanna",
                "the history of susanna",
            ],
            BookId::BelAndTheDragon => &["bel and the dragon", "bel", "bel and draco"],
            BookId::PrayerOfManasses => &[
                "prayer of manasses",
                "prayer of manasseh",
                "the prayer of manasses",
                "prman",
            ],
            BookId::FirstMaccabees => &[
                "1 maccabees",
                "i maccabees",
                "first maccabees",
                "1 macc",
                "1mac",
            ],
            BookId::SecondMaccabees => &[
                "2 maccabees",
                "ii maccabees",
                "second maccabees",
                "2 macc",
                "2mac",
            ],
        }
    }
}

impl fmt::Display for BookId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.canonical_name())
    }
}

/// Resolve a book name (case-insensitive, whitespace-tolerant) to a `BookId`.
///
/// Returns the alias that matched, so the caller can strip it from a
/// reference string.
pub fn resolve_book(input: &str) -> Result<(BookId, usize), BookLookupError> {
    let norm = normalize_book_input(input);
    let mut best: Option<(BookId, usize)> = None;
    for book in BookId::ALL {
        for alias in book.aliases() {
            let a = normalize_book_input(alias);
            if let Some(rest) = norm.strip_prefix(&a) {
                // The alias must end at a word boundary (or the whole string).
                if rest.is_empty() || rest.starts_with(' ') {
                    let matched = a.chars().count();
                    if best.is_none_or(|(_, m)| matched > m) {
                        best = Some((book, matched));
                    }
                }
            }
        }
    }
    match best {
        Some((book, len)) => Ok((book, len)),
        None => Err(BookLookupError::UnknownBook {
            input: input.trim().to_string(),
            suggestions: suggest_books(&norm),
        }),
    }
}

/// Lowercase + NFC + collapse whitespace, for matching user input.
pub fn normalize_book_input(input: &str) -> String {
    use unicode_normalization::UnicodeNormalization;
    let nfc: String = input.nfkc().collect();
    nfc.to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn suggest_books(norm: &str) -> Vec<&'static str> {
    // Suggest books whose name is close to the (book-ish part of the) input:
    // containment or small edit distance.
    let bookish = bookish_part(norm);
    let mut out: Vec<&'static str> = Vec::new();
    for book in BookId::ALL {
        for alias in book.aliases() {
            let a = normalize_book_input(alias);
            let close = (a.len() >= 4 && (bookish.contains(&a) || a.contains(&bookish)))
                || (bookish.len() >= 3 && levenshtein(&bookish, &a) <= 2);
            if close && !out.contains(&book.canonical_name()) {
                out.push(book.canonical_name());
            }
        }
    }
    out.truncate(5);
    out
}

/// The leading book-name part of a reference, e.g. `1 maccabees 3:1` ->
/// `1 maccabees`; `sirah 2:1` -> `sirah`.
fn bookish_part(norm: &str) -> String {
    let mut out: Vec<&str> = Vec::new();
    for tok in norm.split_whitespace() {
        let is_ref =
            tok.bytes().all(|b| b.is_ascii_digit()) || tok.contains(':') || tok.contains('-');
        if is_ref {
            break;
        }
        out.push(tok);
    }
    out.join(" ")
}

/// Classic two-row Levenshtein distance.
fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0usize; b.len() + 1];
    for (i, ca) in a.iter().enumerate() {
        cur[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            cur[j + 1] = (cur[j] + 1).min(prev[j + 1] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BookLookupError {
    UnknownBook {
        input: String,
        suggestions: Vec<&'static str>,
    },
}

impl std::fmt::Display for BookLookupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BookLookupError::UnknownBook { input, suggestions } => {
                write!(f, "unknown book: {input:?}")?;
                if !suggestions.is_empty() {
                    write!(f, " (did you mean: {})", suggestions.join(", "))?;
                }
            }
        }
        Ok(())
    }
}

impl std::error::Error for BookLookupError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_required_aliases() {
        let cases = [
            ("Sirach", BookId::Sirach),
            ("sirach", BookId::Sirach),
            ("SIRACH", BookId::Sirach),
            ("Ecclesiasticus", BookId::Sirach),
            ("Ecclus", BookId::Sirach),
            ("Wisdom", BookId::WisdomOfSolomon),
            ("Wisdom of Solomon", BookId::WisdomOfSolomon),
            ("1 Maccabees", BookId::FirstMaccabees),
            ("2 Maccabees", BookId::SecondMaccabees),
            ("1 macc", BookId::FirstMaccabees),
            ("Tobit", BookId::Tobit),
            ("Judith", BookId::Judith),
            ("1 Esdras", BookId::FirstEsdras),
            ("2 Esdras", BookId::SecondEsdras),
            ("Rest of Esther", BookId::RestOfEsther),
            ("Baruch", BookId::Baruch),
            ("Epistle of Jeremy", BookId::EpistleOfJeremy),
            ("Prayer of Azariah", BookId::PrayerOfAzariah),
            ("Susanna", BookId::Susanna),
            ("Bel and the Dragon", BookId::BelAndTheDragon),
            ("Prayer of Manasses", BookId::PrayerOfManasses),
        ];
        for (input, expected) in cases {
            let (book, _) = resolve_book(input).unwrap_or_else(|e| panic!("{input}: {e}"));
            assert_eq!(book, expected, "input {input:?}");
        }
    }

    #[test]
    fn longest_alias_wins() {
        // "wisdom of solomon" must not match the shorter alias "wisdom" only.
        let (book, len) = resolve_book("wisdom of solomon 2:1").unwrap();
        assert_eq!(book, BookId::WisdomOfSolomon);
        assert_eq!(len, "wisdom of solomon".chars().count());
    }

    #[test]
    fn unknown_book_gives_suggestions() {
        let err = resolve_book("sirah").unwrap_err();
        match err {
            BookLookupError::UnknownBook { suggestions, .. } => {
                assert!(suggestions.contains(&"Sirach"));
            }
        }
    }

    #[test]
    fn canonical_names_are_unique() {
        let mut names: Vec<&str> = BookId::ALL.iter().map(|b| b.canonical_name()).collect();
        let n = names.len();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), n);
    }
}
