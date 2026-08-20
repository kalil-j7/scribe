//! Text witnesses: language, tradition, and dataset identity.

use serde::{Deserialize, Serialize};

/// Which text tradition a witness belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextTradition {
    /// KJV 1769 (Old Testament, Apocrypha, and New Testament).
    KjvApocrypha,
    /// Greek Septuagint (Rahlfs edition via the LXXMorph corpus).
    Septuagint,
}

/// Language of a witness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    English,
    Greek,
}

impl Language {
    pub fn label(self) -> &'static str {
        match self {
            Language::English => "ENGLISH",
            Language::Greek => "GREEK",
        }
    }
}

/// Stable identifier for an installed witness/dataset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WitnessId {
    /// King James Version (1769), full KJVA canon.
    KjvApocrypha,
    /// Greek Septuagint, Apocrypha books (Rahlfs via LXXMorph).
    Lxx,
}

impl WitnessId {
    pub const ALL: [WitnessId; 2] = [WitnessId::KjvApocrypha, WitnessId::Lxx];

    pub fn meta(self) -> WitnessMeta {
        match self {
            WitnessId::KjvApocrypha => WitnessMeta {
                id: self,
                title: "KJV (1769)".to_string(),
                language: Language::English,
                tradition: TextTradition::KjvApocrypha,
            },
            WitnessId::Lxx => WitnessMeta {
                id: self,
                title: "Greek (LXX)".to_string(),
                language: Language::Greek,
                tradition: TextTradition::Septuagint,
            },
        }
    }

    /// File name of the JSONL store for this witness.
    pub fn store_file(self) -> &'static str {
        match self {
            WitnessId::KjvApocrypha => "kjva.jsonl",
            WitnessId::Lxx => "lxx.jsonl",
        }
    }

    /// File name of the binary cache for this witness.
    pub fn cache_file(self) -> &'static str {
        match self {
            WitnessId::KjvApocrypha => "kjva.cache",
            WitnessId::Lxx => "lxx.cache",
        }
    }

    /// User-facing dataset name for `scribe data install` / `doctor`.
    pub fn dataset_name(self) -> &'static str {
        match self {
            WitnessId::KjvApocrypha => "kjva",
            WitnessId::Lxx => "lxx",
        }
    }
}

/// Human-facing description of a witness.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WitnessMeta {
    pub id: WitnessId,
    pub title: String,
    pub language: Language,
    pub tradition: TextTradition,
}
