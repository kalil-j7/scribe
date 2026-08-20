//! The data-source boundary.
//!
//! Everything above this trait (CLI, application services) depends only on
//! domain types. A concrete source — the native store today, a SWORD adapter
//! tomorrow — is interchangeable behind this trait.

use crate::domain::book::BookId;
use crate::domain::passage::{BookInfo, Chapter, DatasetInfo, Passage};
use crate::domain::reference::{ChapterNumber, PassageRef};
use crate::domain::search::{SearchHit, SearchQuery};
use crate::domain::witness::WitnessId;
use crate::error::ScribeError;

/// A read-only source of Scripture text.
pub trait ScriptureSource {
    /// Look up a passage (verse range or whole chapter) in one witness.
    fn passage(&self, reference: &PassageRef, witness: WitnessId) -> Result<Passage, ScribeError>;

    /// Look up a whole chapter in one witness.
    fn chapter(
        &self,
        book: BookId,
        chapter: ChapterNumber,
        witness: WitnessId,
    ) -> Result<Chapter, ScribeError>;

    /// Full-text search over one witness.
    fn search(&self, query: &SearchQuery) -> Result<Vec<SearchHit>, ScribeError>;

    /// Per-chapter verse counts for a book in a witness (used for validation
    /// and `scribe books`).
    fn book_info(&self, book: BookId, witness: WitnessId) -> Result<BookInfo, ScribeError>;

    /// Which datasets are installed, with verse counts.
    fn datasets(&self) -> Vec<DatasetInfo>;
}
