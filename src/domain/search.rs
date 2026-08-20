//! Search domain types.

use serde::{Deserialize, Serialize};

use super::book::{BookId, ScriptureCorpus};
use super::reference::{ChapterNumber, VerseNumber};
use super::witness::WitnessId;

/// A full-text search request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchQuery {
    /// The normalized search terms (all must match a verse).
    pub terms: Vec<String>,
    /// Restrict to one book.
    pub book: Option<BookId>,
    /// Restrict to one KJV corpus division.
    pub corpus: Option<ScriptureCorpus>,
    /// Which witness to search (defaults to complete KJV).
    pub witness: WitnessId,
    /// Maximum number of hits to return.
    pub limit: usize,
}

/// One search hit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchHit {
    pub witness: WitnessId,
    pub book: BookId,
    pub chapter: ChapterNumber,
    pub verse: VerseNumber,
    pub text: String,
    /// Number of query-term occurrences in this verse (ranking score).
    pub score: u32,
}
