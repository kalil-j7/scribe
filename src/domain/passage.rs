//! Scripture text: witnesses, tokens, verses, passages, chapters.

use serde::{Deserialize, Serialize};

use super::book::BookId;
use super::reference::{ChapterNumber, PassageRef, VerseNumber};
use super::witness::{WitnessId, WitnessMeta};

/// A single word/token of a verse, with optional lemma and morphology.
///
/// This is the extension point for the future word-study features
/// (`scribe word πειρασμός`, `scribe occurrences ...`, morphology, alignment).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Token {
    /// The surface form as printed in the witness (e.g. `προσέρχῃ`).
    pub surface: String,
    /// Normalized form used for matching (NFC, lowercase, accent-insensitive).
    pub normalized: String,
    /// Dictionary lemma, when the source provides one (Greek witness).
    pub lemma: Option<String>,
    /// Morphological parse tag, when the source provides one (Greek witness).
    pub morph: Option<String>,
}

/// One verse of one witness, in our domain shape.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptureText {
    pub witness: WitnessId,
    pub book: BookId,
    pub chapter: ChapterNumber,
    pub verse: VerseNumber,
    pub text: String,
    pub tokens: Vec<Token>,
}

/// A passage (one or more verses of one witness).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Passage {
    pub reference: PassageRef,
    pub witness: WitnessId,
    pub verses: Vec<ScriptureText>,
}

/// A whole chapter of one witness.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Chapter {
    pub book: BookId,
    pub chapter: ChapterNumber,
    pub witness: WitnessId,
    pub verses: Vec<ScriptureText>,
}

/// Describes one installed dataset (shown by `scribe doctor` / `scribe data status`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatasetInfo {
    pub witness: WitnessMeta,
    pub available: bool,
    pub verses: u64,
    pub path: Option<String>,
}

/// Per-chapter verse counts for one book of one witness.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BookInfo {
    pub book: BookId,
    pub witness: WitnessId,
    /// (chapter, verse_count) pairs, in order.
    pub chapters: Vec<(ChapterNumber, u16)>,
}

/// Text quality metadata attached to every imported dataset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provenance {
    pub dataset: String,
    pub source: String,
    pub edition: String,
    pub license: String,
    pub redistribution: String,
    pub commercial_use: String,
    pub imported_at: String,
}
