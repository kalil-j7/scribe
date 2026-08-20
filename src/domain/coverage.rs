//! Declared coverage of Scribe's KJV Apocrypha books in the selected Greek
//! witness.  This is deliberately about *identity and usable reference
//! mapping*, not merely whether a similarly named CCAT file exists.

use serde::{Deserialize, Serialize};

use super::book::BookId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoverageStatus {
    Full,
    Partial,
    Unavailable,
    VersificationConflict,
}

impl CoverageStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Partial => "partial",
            Self::Unavailable => "unavailable",
            Self::VersificationConflict => "versification conflict",
        }
    }

    pub fn supports_lookup(self) -> bool {
        matches!(self, Self::Full | Self::Partial)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WitnessCoverage {
    pub book: BookId,
    pub status: CoverageStatus,
    /// Brief user-facing source/mapping note. Kept static so `books` remains
    /// fast and works even before Greek data is installed.
    pub note: &'static str,
}

pub const LXX_COVERAGE: [WitnessCoverage; 15] = [
    WitnessCoverage {
        book: BookId::FirstEsdras,
        status: CoverageStatus::Partial,
        note: "CCAT 1Esdr is the same work; verse divisions diverge in several chapters.",
    },
    WitnessCoverage {
        book: BookId::SecondEsdras,
        status: CoverageStatus::Unavailable,
        note: "CCAT 2Esdr is Greek Ezra-Nehemiah, not KJV 2 Esdras / 4 Ezra.",
    },
    WitnessCoverage {
        book: BookId::Tobit,
        status: CoverageStatus::Partial,
        note: "Selected BA recension; KJV and source verse divisions differ.",
    },
    WitnessCoverage {
        book: BookId::Judith,
        status: CoverageStatus::Partial,
        note: "One source/KJV verse-division difference remains.",
    },
    WitnessCoverage {
        book: BookId::RestOfEsther,
        status: CoverageStatus::VersificationConflict,
        note: "Greek Esther additions use lettered segments; no safe KJV crosswalk is enabled.",
    },
    WitnessCoverage {
        book: BookId::WisdomOfSolomon,
        status: CoverageStatus::Partial,
        note: "One source/KJV verse-division difference remains.",
    },
    WitnessCoverage {
        book: BookId::Sirach,
        status: CoverageStatus::Partial,
        note: "Rahlfs text; several later source verse divisions diverge (Sirach 2 is aligned).",
    },
    WitnessCoverage {
        book: BookId::Baruch,
        status: CoverageStatus::Partial,
        note: "One source/KJV verse-division difference remains.",
    },
    WitnessCoverage {
        book: BookId::EpistleOfJeremy,
        status: CoverageStatus::Full,
        note: "Separate Scribe work; CCAT EpJer numbering is normalized to 1:1–73.",
    },
    WitnessCoverage {
        book: BookId::PrayerOfAzariah,
        status: CoverageStatus::Full,
        note: "Daniel Theodotion 3:24–91, mapped to KJV Prayer of Azariah 1:1–68.",
    },
    WitnessCoverage {
        book: BookId::Susanna,
        status: CoverageStatus::Full,
        note: "Theodotion Susanna 1–64.",
    },
    WitnessCoverage {
        book: BookId::BelAndTheDragon,
        status: CoverageStatus::Full,
        note: "Theodotion Bel 1–42.",
    },
    WitnessCoverage {
        book: BookId::PrayerOfManasses,
        status: CoverageStatus::Full,
        note: "Odes 12:1–15 combined into KJV's single printed verse.",
    },
    WitnessCoverage {
        book: BookId::FirstMaccabees,
        status: CoverageStatus::Full,
        note: "CCAT 1Macc; matching KJV reference grid.",
    },
    WitnessCoverage {
        book: BookId::SecondMaccabees,
        status: CoverageStatus::Full,
        note: "CCAT 2Macc; matching KJV reference grid.",
    },
];

pub fn lxx_coverage(book: BookId) -> &'static WitnessCoverage {
    LXX_COVERAGE
        .iter()
        .find(|c| c.book == book)
        .expect("all BookId values covered")
}

/// `compare` requires an explicitly verified one-to-one reference mapping.
/// The source grid is safe for all full adapters and for the long-established
/// Sirach 2 reference used by the product's primary workflow.
pub fn lxx_compare_supported(book: BookId, chapter: u16, start: u16, end: u16) -> bool {
    matches!(lxx_coverage(book).status, CoverageStatus::Full)
        || (book == BookId::Sirach && chapter == 2 && start >= 1 && end <= 18)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_kjv_book_has_exactly_one_lxx_status() {
        assert_eq!(LXX_COVERAGE.len(), BookId::ALL.len());
        for book in BookId::ALL {
            assert_eq!(LXX_COVERAGE.iter().filter(|c| c.book == book).count(), 1);
        }
        assert_eq!(
            lxx_coverage(BookId::SecondEsdras).status,
            CoverageStatus::Unavailable
        );
    }
}
