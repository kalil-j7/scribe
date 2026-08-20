//! Typed errors for Scribe.

use crate::domain::reference::ReferenceParseError;

#[derive(Debug, thiserror::Error)]
pub enum ScribeError {
    #[error(transparent)]
    Reference(#[from] ReferenceParseError),

    #[error("no Greek data installed for {book}: run `scribe data install lxx`")]
    GreekNotInstalled { book: String },

    #[error("Greek data is not installed\nhint: run `scribe data install lxx`")]
    GreekDataNotInstalled,

    #[error("Greek coverage for {book} is {status} in the installed LXX source: {note}")]
    GreekCoverageUnavailable {
        book: String,
        status: String,
        note: String,
    },

    #[error("Greek compare is not enabled for {book}: {note}")]
    GreekCompareUnavailable { book: String, note: String },

    #[error("no Greek word or lemma matching {query:?} was found in the installed corpus")]
    LemmaNotFound { query: String },

    #[error("{surface:?} maps to multiple lemmas:\n{list}")]
    AmbiguousLemma { surface: String, list: String },

    #[error("no English Apocrypha data installed: run `scribe setup`")]
    KjvaNotInstalled,

    #[error("book {book:?} is not present in the {witness} witness")]
    BookNotInWitness { book: String, witness: String },

    #[error("{book} has no chapter {chapter} (max {max})")]
    ChapterOutOfRange {
        book: String,
        chapter: u16,
        max: u16,
    },

    #[error("{book} {chapter} has no verse {verse} (max {max})")]
    VerseOutOfRange {
        book: String,
        chapter: u16,
        verse: u16,
        max: u16,
    },

    #[error("store is missing or corrupt at {path}: {detail}")]
    StoreCorrupt { path: String, detail: String },

    #[error("cannot find data directory (is SCRIBE_DATA_DIR set?)")]
    NoDataDir,

    #[error("I/O error at {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("network error downloading {url}: {detail}")]
    Download { url: String, detail: String },

    #[error("invalid dataset name {name:?} (expected `kjva`, `lxx` or `all`)")]
    UnknownDataset { name: String },

    #[error("dataset {name:?} is not installed")]
    NotInstalled { name: String },

    #[error("import failed for {dataset}: {detail}")]
    ImportFailed { dataset: String, detail: String },

    #[error("{0}")]
    Other(String),
}

impl From<std::io::Error> for ScribeError {
    fn from(e: std::io::Error) -> Self {
        ScribeError::Io {
            path: "?".to_string(),
            source: e,
        }
    }
}

pub type Result<T> = std::result::Result<T, ScribeError>;
