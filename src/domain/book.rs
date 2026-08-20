//! Stable KJV canon order, display names, corpus membership, and aliases.
//!
//! The bundled source is CrossWire's KJVA OSIS: canonical Old Testament,
//! KJV Apocrypha, and canonical New Testament.  Book identity is Scribe's
//! own model and never follows a witness's incidental layout.

use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BookId {
    Genesis,
    Exodus,
    Leviticus,
    Numbers,
    Deuteronomy,
    Joshua,
    Judges,
    Ruth,
    FirstSamuel,
    SecondSamuel,
    FirstKings,
    SecondKings,
    FirstChronicles,
    SecondChronicles,
    Ezra,
    Nehemiah,
    Esther,
    Job,
    Psalms,
    Proverbs,
    Ecclesiastes,
    SongOfSolomon,
    Isaiah,
    Jeremiah,
    Lamentations,
    Ezekiel,
    Daniel,
    Hosea,
    Joel,
    Amos,
    Obadiah,
    Jonah,
    Micah,
    Nahum,
    Habakkuk,
    Zephaniah,
    Haggai,
    Zechariah,
    Malachi,
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
    Matthew,
    Mark,
    Luke,
    John,
    Acts,
    Romans,
    FirstCorinthians,
    SecondCorinthians,
    Galatians,
    Ephesians,
    Philippians,
    Colossians,
    FirstThessalonians,
    SecondThessalonians,
    FirstTimothy,
    SecondTimothy,
    Titus,
    Philemon,
    Hebrews,
    James,
    FirstPeter,
    SecondPeter,
    FirstJohn,
    SecondJohn,
    ThirdJohn,
    Jude,
    Revelation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScriptureCorpus {
    OldTestament,
    Apocrypha,
    NewTestament,
}

impl ScriptureCorpus {
    pub fn label(self) -> &'static str {
        match self {
            Self::OldTestament => "Old Testament",
            Self::Apocrypha => "Apocrypha",
            Self::NewTestament => "New Testament",
        }
    }
    pub fn key(self) -> &'static str {
        match self {
            Self::OldTestament => "ot",
            Self::Apocrypha => "apocrypha",
            Self::NewTestament => "nt",
        }
    }
    pub fn parse(input: &str) -> Option<Self> {
        match input.trim().to_ascii_lowercase().as_str() {
            "ot" | "old" | "old_testament" | "old-testament" => Some(Self::OldTestament),
            "apocrypha" | "apoc" => Some(Self::Apocrypha),
            "nt" | "new" | "new_testament" | "new-testament" => Some(Self::NewTestament),
            _ => None,
        }
    }
}

impl BookId {
    pub const ALL: [BookId; 81] = [
        Self::Genesis,
        Self::Exodus,
        Self::Leviticus,
        Self::Numbers,
        Self::Deuteronomy,
        Self::Joshua,
        Self::Judges,
        Self::Ruth,
        Self::FirstSamuel,
        Self::SecondSamuel,
        Self::FirstKings,
        Self::SecondKings,
        Self::FirstChronicles,
        Self::SecondChronicles,
        Self::Ezra,
        Self::Nehemiah,
        Self::Esther,
        Self::Job,
        Self::Psalms,
        Self::Proverbs,
        Self::Ecclesiastes,
        Self::SongOfSolomon,
        Self::Isaiah,
        Self::Jeremiah,
        Self::Lamentations,
        Self::Ezekiel,
        Self::Daniel,
        Self::Hosea,
        Self::Joel,
        Self::Amos,
        Self::Obadiah,
        Self::Jonah,
        Self::Micah,
        Self::Nahum,
        Self::Habakkuk,
        Self::Zephaniah,
        Self::Haggai,
        Self::Zechariah,
        Self::Malachi,
        Self::FirstEsdras,
        Self::SecondEsdras,
        Self::Tobit,
        Self::Judith,
        Self::RestOfEsther,
        Self::WisdomOfSolomon,
        Self::Sirach,
        Self::Baruch,
        Self::EpistleOfJeremy,
        Self::PrayerOfAzariah,
        Self::Susanna,
        Self::BelAndTheDragon,
        Self::PrayerOfManasses,
        Self::FirstMaccabees,
        Self::SecondMaccabees,
        Self::Matthew,
        Self::Mark,
        Self::Luke,
        Self::John,
        Self::Acts,
        Self::Romans,
        Self::FirstCorinthians,
        Self::SecondCorinthians,
        Self::Galatians,
        Self::Ephesians,
        Self::Philippians,
        Self::Colossians,
        Self::FirstThessalonians,
        Self::SecondThessalonians,
        Self::FirstTimothy,
        Self::SecondTimothy,
        Self::Titus,
        Self::Philemon,
        Self::Hebrews,
        Self::James,
        Self::FirstPeter,
        Self::SecondPeter,
        Self::FirstJohn,
        Self::SecondJohn,
        Self::ThirdJohn,
        Self::Jude,
        Self::Revelation,
    ];
    pub fn corpus(self) -> ScriptureCorpus {
        match self {
            Self::FirstEsdras
            | Self::SecondEsdras
            | Self::Tobit
            | Self::Judith
            | Self::RestOfEsther
            | Self::WisdomOfSolomon
            | Self::Sirach
            | Self::Baruch
            | Self::EpistleOfJeremy
            | Self::PrayerOfAzariah
            | Self::Susanna
            | Self::BelAndTheDragon
            | Self::PrayerOfManasses
            | Self::FirstMaccabees
            | Self::SecondMaccabees => ScriptureCorpus::Apocrypha,
            Self::Matthew
            | Self::Mark
            | Self::Luke
            | Self::John
            | Self::Acts
            | Self::Romans
            | Self::FirstCorinthians
            | Self::SecondCorinthians
            | Self::Galatians
            | Self::Ephesians
            | Self::Philippians
            | Self::Colossians
            | Self::FirstThessalonians
            | Self::SecondThessalonians
            | Self::FirstTimothy
            | Self::SecondTimothy
            | Self::Titus
            | Self::Philemon
            | Self::Hebrews
            | Self::James
            | Self::FirstPeter
            | Self::SecondPeter
            | Self::FirstJohn
            | Self::SecondJohn
            | Self::ThirdJohn
            | Self::Jude
            | Self::Revelation => ScriptureCorpus::NewTestament,
            _ => ScriptureCorpus::OldTestament,
        }
    }
    pub fn canonical_name(self) -> &'static str {
        match self {
            Self::Genesis => "Genesis",
            Self::Exodus => "Exodus",
            Self::Leviticus => "Leviticus",
            Self::Numbers => "Numbers",
            Self::Deuteronomy => "Deuteronomy",
            Self::Joshua => "Joshua",
            Self::Judges => "Judges",
            Self::Ruth => "Ruth",
            Self::FirstSamuel => "1 Samuel",
            Self::SecondSamuel => "2 Samuel",
            Self::FirstKings => "1 Kings",
            Self::SecondKings => "2 Kings",
            Self::FirstChronicles => "1 Chronicles",
            Self::SecondChronicles => "2 Chronicles",
            Self::Ezra => "Ezra",
            Self::Nehemiah => "Nehemiah",
            Self::Esther => "Esther",
            Self::Job => "Job",
            Self::Psalms => "Psalms",
            Self::Proverbs => "Proverbs",
            Self::Ecclesiastes => "Ecclesiastes",
            Self::SongOfSolomon => "Song of Solomon",
            Self::Isaiah => "Isaiah",
            Self::Jeremiah => "Jeremiah",
            Self::Lamentations => "Lamentations",
            Self::Ezekiel => "Ezekiel",
            Self::Daniel => "Daniel",
            Self::Hosea => "Hosea",
            Self::Joel => "Joel",
            Self::Amos => "Amos",
            Self::Obadiah => "Obadiah",
            Self::Jonah => "Jonah",
            Self::Micah => "Micah",
            Self::Nahum => "Nahum",
            Self::Habakkuk => "Habakkuk",
            Self::Zephaniah => "Zephaniah",
            Self::Haggai => "Haggai",
            Self::Zechariah => "Zechariah",
            Self::Malachi => "Malachi",
            Self::FirstEsdras => "1 Esdras",
            Self::SecondEsdras => "2 Esdras",
            Self::Tobit => "Tobit",
            Self::Judith => "Judith",
            Self::RestOfEsther => "Rest of Esther",
            Self::WisdomOfSolomon => "Wisdom of Solomon",
            Self::Sirach => "Sirach",
            Self::Baruch => "Baruch",
            Self::EpistleOfJeremy => "Epistle of Jeremy",
            Self::PrayerOfAzariah => "Prayer of Azariah",
            Self::Susanna => "Susanna",
            Self::BelAndTheDragon => "Bel and the Dragon",
            Self::PrayerOfManasses => "Prayer of Manasses",
            Self::FirstMaccabees => "1 Maccabees",
            Self::SecondMaccabees => "2 Maccabees",
            Self::Matthew => "Matthew",
            Self::Mark => "Mark",
            Self::Luke => "Luke",
            Self::John => "John",
            Self::Acts => "Acts",
            Self::Romans => "Romans",
            Self::FirstCorinthians => "1 Corinthians",
            Self::SecondCorinthians => "2 Corinthians",
            Self::Galatians => "Galatians",
            Self::Ephesians => "Ephesians",
            Self::Philippians => "Philippians",
            Self::Colossians => "Colossians",
            Self::FirstThessalonians => "1 Thessalonians",
            Self::SecondThessalonians => "2 Thessalonians",
            Self::FirstTimothy => "1 Timothy",
            Self::SecondTimothy => "2 Timothy",
            Self::Titus => "Titus",
            Self::Philemon => "Philemon",
            Self::Hebrews => "Hebrews",
            Self::James => "James",
            Self::FirstPeter => "1 Peter",
            Self::SecondPeter => "2 Peter",
            Self::FirstJohn => "1 John",
            Self::SecondJohn => "2 John",
            Self::ThirdJohn => "3 John",
            Self::Jude => "Jude",
            Self::Revelation => "Revelation",
        }
    }
    pub fn from_canonical_name(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|book| book.canonical_name() == name)
    }
    pub fn aliases(self) -> &'static [&'static str] {
        match self {
            Self::Genesis => &["genesis", "gen"],
            Self::Exodus => &["exodus", "exod", "exo"],
            Self::Leviticus => &["leviticus", "lev"],
            Self::Numbers => &["numbers", "num"],
            Self::Deuteronomy => &["deuteronomy", "deut"],
            Self::Joshua => &["joshua", "josh"],
            Self::Judges => &["judges", "judg"],
            Self::Ruth => &["ruth"],
            Self::FirstSamuel => &["1 samuel", "i samuel", "first samuel", "1 sam"],
            Self::SecondSamuel => &["2 samuel", "ii samuel", "second samuel", "2 sam"],
            Self::FirstKings => &["1 kings", "i kings", "first kings", "1 kgs"],
            Self::SecondKings => &["2 kings", "ii kings", "second kings", "2 kgs"],
            Self::FirstChronicles => &[
                "1 chronicles",
                "i chronicles",
                "first chronicles",
                "1 chron",
            ],
            Self::SecondChronicles => &[
                "2 chronicles",
                "ii chronicles",
                "second chronicles",
                "2 chron",
            ],
            Self::Ezra => &["ezra"],
            Self::Nehemiah => &["nehemiah", "neh"],
            Self::Esther => &["esther", "est"],
            Self::Job => &["job"],
            Self::Psalms => &["psalms", "psalm", "ps"],
            Self::Proverbs => &["proverbs", "proverb", "prov", "pr"],
            Self::Ecclesiastes => &["ecclesiastes", "eccl"],
            Self::SongOfSolomon => &["song of solomon", "song", "song of songs"],
            Self::Isaiah => &["isaiah", "isa"],
            Self::Jeremiah => &["jeremiah", "jer"],
            Self::Lamentations => &["lamentations", "lam"],
            Self::Ezekiel => &["ezekiel", "ezek"],
            Self::Daniel => &["daniel", "dan"],
            Self::Hosea => &["hosea", "hos"],
            Self::Joel => &["joel"],
            Self::Amos => &["amos"],
            Self::Obadiah => &["obadiah", "obad"],
            Self::Jonah => &["jonah", "jon"],
            Self::Micah => &["micah", "mic"],
            Self::Nahum => &["nahum", "nah"],
            Self::Habakkuk => &["habakkuk", "hab"],
            Self::Zephaniah => &["zephaniah", "zeph"],
            Self::Haggai => &["haggai", "hag"],
            Self::Zechariah => &["zechariah", "zech"],
            Self::Malachi => &["malachi", "mal"],
            Self::FirstEsdras => &["1 esdras", "i esdras", "first esdras", "1 esd", "esdras 1"],
            Self::SecondEsdras => &[
                "2 esdras",
                "ii esdras",
                "second esdras",
                "2 esd",
                "esdras 2",
            ],
            Self::Tobit => &["tobit", "tob", "tobias"],
            Self::Judith => &["judith", "jdt"],
            Self::RestOfEsther => &[
                "rest of esther",
                "additions to esther",
                "additions of esther",
                "add esther",
            ],
            Self::WisdomOfSolomon => &["wisdom of solomon", "wisdom", "wis"],
            Self::Sirach => &["sirach", "ecclesiasticus", "ecclus", "sir"],
            Self::Baruch => &["baruch", "bar"],
            Self::EpistleOfJeremy => &[
                "epistle of jeremy",
                "epistle of jeremiah",
                "jeremy's epistle",
            ],
            Self::PrayerOfAzariah => &[
                "prayer of azariah",
                "prayer of azarias",
                "song of the three children",
                "azariah",
            ],
            Self::Susanna => &["susanna", "sus"],
            Self::BelAndTheDragon => &["bel and the dragon", "bel"],
            Self::PrayerOfManasses => &["prayer of manasses", "prayer of manasseh", "prman"],
            Self::FirstMaccabees => &[
                "1 maccabees",
                "i maccabees",
                "first maccabees",
                "1 macc",
                "1mac",
            ],
            Self::SecondMaccabees => &[
                "2 maccabees",
                "ii maccabees",
                "second maccabees",
                "2 macc",
                "2mac",
            ],
            Self::Matthew => &["matthew", "matt"],
            Self::Mark => &["mark", "mrk"],
            Self::Luke => &["luke", "luk"],
            Self::John => &["john", "jhn"],
            Self::Acts => &["acts", "act"],
            Self::Romans => &["romans", "rom"],
            Self::FirstCorinthians => &[
                "1 corinthians",
                "i corinthians",
                "first corinthians",
                "1 cor",
            ],
            Self::SecondCorinthians => &[
                "2 corinthians",
                "ii corinthians",
                "second corinthians",
                "2 cor",
            ],
            Self::Galatians => &["galatians", "gal"],
            Self::Ephesians => &["ephesians", "eph"],
            Self::Philippians => &["philippians", "phil"],
            Self::Colossians => &["colossians", "col"],
            Self::FirstThessalonians => &[
                "1 thessalonians",
                "i thessalonians",
                "first thessalonians",
                "1 thess",
            ],
            Self::SecondThessalonians => &[
                "2 thessalonians",
                "ii thessalonians",
                "second thessalonians",
                "2 thess",
            ],
            Self::FirstTimothy => &["1 timothy", "i timothy", "first timothy", "1 tim"],
            Self::SecondTimothy => &["2 timothy", "ii timothy", "second timothy", "2 tim"],
            Self::Titus => &["titus", "tit"],
            Self::Philemon => &["philemon", "phlm"],
            Self::Hebrews => &["hebrews", "heb"],
            Self::James => &["james", "jas"],
            Self::FirstPeter => &["1 peter", "i peter", "first peter", "1 pet"],
            Self::SecondPeter => &["2 peter", "ii peter", "second peter", "2 pet"],
            Self::FirstJohn => &["1 john", "i john", "first john", "1 jn"],
            Self::SecondJohn => &["2 john", "ii john", "second john", "2 jn"],
            Self::ThirdJohn => &["3 john", "iii john", "third john", "3 jn"],
            Self::Jude => &["jude"],
            Self::Revelation => &["revelation", "rev", "apocalypse"],
        }
    }
}

impl fmt::Display for BookId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.canonical_name())
    }
}

pub fn resolve_book(input: &str) -> Result<(BookId, usize), BookLookupError> {
    let norm = normalize_book_input(input);
    let mut best = None;
    for book in BookId::ALL {
        for alias in book.aliases() {
            let a = normalize_book_input(alias);
            if let Some(rest) = norm.strip_prefix(&a) {
                if rest.is_empty() || rest.starts_with(' ') {
                    let matched = a.chars().count();
                    if best.is_none_or(|(_, m)| matched > m) {
                        best = Some((book, matched));
                    }
                }
            }
        }
    }
    best.ok_or_else(|| BookLookupError::UnknownBook {
        input: input.trim().to_string(),
        suggestions: suggest_books(&norm),
    })
}
pub fn normalize_book_input(input: &str) -> String {
    use unicode_normalization::UnicodeNormalization;
    let nfc: String = input.nfkc().collect();
    nfc.to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
fn suggest_books(norm: &str) -> Vec<&'static str> {
    let bookish = bookish_part(norm);
    let mut out = Vec::new();
    for book in BookId::ALL {
        for alias in book.aliases() {
            let a = normalize_book_input(alias);
            if ((a.len() >= 4 && (bookish.contains(&a) || a.contains(&bookish)))
                || (bookish.len() >= 3 && levenshtein(&bookish, &a) <= 2))
                && !out.contains(&book.canonical_name())
            {
                out.push(book.canonical_name());
            }
        }
    }
    out.truncate(5);
    out
}
fn bookish_part(norm: &str) -> String {
    norm.split_whitespace()
        .take_while(|tok| {
            !(tok.bytes().all(|b| b.is_ascii_digit()) || tok.contains(':') || tok.contains('-'))
        })
        .collect::<Vec<_>>()
        .join(" ")
}
fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0usize; b.len() + 1];
    for (i, ca) in a.iter().enumerate() {
        cur[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
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
impl fmt::Display for BookLookupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownBook { input, suggestions } => {
                write!(f, "unknown book: {input:?}")?;
                if !suggestions.is_empty() {
                    write!(f, " (did you mean: {})", suggestions.join(", "))?;
                }
                Ok(())
            }
        }
    }
}
impl std::error::Error for BookLookupError {}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn canonical_names_are_unique() {
        let mut names: Vec<_> = BookId::ALL.iter().map(|b| b.canonical_name()).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), BookId::ALL.len());
    }
    #[test]
    fn resolves_required_aliases() {
        for (input, expected) in [
            ("gen 1:1", BookId::Genesis),
            ("psalm 23", BookId::Psalms),
            ("1 cor 13", BookId::FirstCorinthians),
            ("1 esdras 1", BookId::FirstEsdras),
            ("revelation 22", BookId::Revelation),
        ] {
            assert_eq!(resolve_book(input).unwrap().0, expected);
        }
    }
}
