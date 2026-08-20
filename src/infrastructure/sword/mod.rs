//! The SWORD adapter seam.
//!
//! This module exists so that a real SWORD backend can be added later without
//! touching the CLI, application, or domain layers. Everything below the
//! [`domain::source::ScriptureSource`] boundary is replaceable.
//!
//! # Evaluation of `rsword_chirho` 0.3.0 (2026-08)
//!
//! Before choosing data paths, `rsword_chirho` was spike-tested against the
//! official CrossWire modules for the exact datasets this milestone needs
//! (see `docs/sword-evaluation.md` for the full transcript). Results:
//!
//! * **KJVA module** (KJV 1769 + Apocrypha): the module declares
//!   `Versification=KJVA`, which `rsword_chirho` does not register. Reading
//!   `Sirach 2:1` / `Tobit 1:1` / `1 Maccabees 3:1` fails with
//!   `InvalidVerseReferenceChirho`; only KJV-canon books (e.g. Genesis 1:1)
//!   resolve. **Blocker for KJVA.**
//! * **LXX module** (Septuagint, Rahlfs): the module's internal versification
//!   differs from `rsword_chirho`'s built-in LXX canon (the module's own conf
//!   warns about this). Verse lookups return *wrong verses* (e.g. `Sirach 2:1`
//!   returns Wisdom 17 text, `Tobit 1:1` returns Judith 14:5).
//!   **Blocker for LXX.**
//!
//! Per the milestone rules, Scribe does **not** burn time repairing somebody
//! else's library. Instead it ships a native store populated from verified,
//! license-clean text sources (bundled KJV 1769 Apocrypha OSIS text; Greek
//! LXXMorph/Rahlfs corpus downloaded on demand). The seam below is the exact
//! place a working SWORD importer would plug in later.

use crate::domain::book::BookId;
use crate::domain::passage::ScriptureText;
use crate::domain::witness::WitnessId;
use crate::error::Result;

/// A source that can hand over raw verse entries for import.
///
/// Implementations would wrap e.g. a SWORD module manager (`rsword_chirho`,
/// libsword, ...) and immediately convert entries into [`ScriptureText`]
/// domain values — no SWORD type ever crosses this boundary.
#[allow(dead_code)]
pub trait ModuleImporter {
    /// Read every verse of `book` from the underlying module.
    fn read_book(&self, witness: WitnessId, book: BookId) -> Result<Vec<ScriptureText>>;

    /// Human-readable description of the backend.
    fn describe(&self) -> &'static str;
}
