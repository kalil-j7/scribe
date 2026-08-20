//! Data-directory resolution.
//!
//! Resolution order:
//! 1. `SCRIBE_DATA_DIR` environment variable (used by tests and power users).
//! 2. The platform data directory, e.g. `~/Library/Application Support/scribe`
//!    (macOS), `~/.local/share/scribe` (Linux), `%LOCALAPPDATA%\scribe` (Windows).

use std::path::PathBuf;

use crate::error::{Result, ScribeError};

pub fn data_dir() -> Result<PathBuf> {
    if let Ok(dir) = std::env::var("SCRIBE_DATA_DIR") {
        if !dir.trim().is_empty() {
            return Ok(PathBuf::from(dir));
        }
    }
    let dirs =
        directories::ProjectDirs::from("dev", "scribe", "scribe").ok_or(ScribeError::NoDataDir)?;
    Ok(dirs.data_dir().to_path_buf())
}
