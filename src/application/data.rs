//! Data management: setup, install, uninstall, status, doctor.

use std::fs;

use crate::cli::DataAction;
use crate::domain::source::ScriptureSource;
use crate::domain::witness::WitnessId;
use crate::error::{Result, ScribeError};
use crate::infrastructure::importer::{download_lxx, import_kjva, import_lxx_morph};
use crate::infrastructure::store::{read_provenance, Store};
use crate::output;

use super::paths::data_dir;

/// Ensure the bundled KJV Apocrypha dataset is installed (fast, offline).
pub fn ensure_kjva(dir: &std::path::Path) -> Result<()> {
    let store_file = dir.join("store").join(WitnessId::KjvApocrypha.store_file());
    if store_file.exists() {
        return Ok(());
    }
    import_kjva(dir)?;
    Ok(())
}

/// `scribe setup` — import the bundled KJV Apocrypha data.
pub fn run_setup(force: bool) -> Result<()> {
    let dir = data_dir()?;
    let store_file = dir.join("store").join(WitnessId::KjvApocrypha.store_file());
    if store_file.exists() && !force {
        println!(
            "KJV Apocrypha data is already installed at {}",
            store_file.display()
        );
        println!("Use `scribe setup --force` to re-import.");
        return Ok(());
    }
    let report = import_kjva(&dir)?;
    println!(
        "Installed {} verses of KJV Apocrypha (1769) -> {}",
        report.verses,
        report.store_path.display()
    );
    println!("Greek data is separate: run `scribe data install lxx` to add it.");
    Ok(())
}

/// `scribe doctor` — report installation status.
pub fn run_doctor(json: bool) -> Result<()> {
    let dir = data_dir()?;
    let store = Store::open(&dir)?;
    let mut datasets = Vec::new();
    for w in WitnessId::ALL {
        datasets.push(crate::domain::passage::DatasetInfo {
            witness: w.meta(),
            available: store.count_verses(w) > 0 || dir.join("store").join(w.store_file()).exists(),
            verses: store.count_verses(w),
            path: if dir.join("store").join(w.store_file()).exists() {
                Some(dir.join("store").join(w.store_file()).display().to_string())
            } else {
                None
            },
        });
    }
    let version = env!("CARGO_PKG_VERSION");
    if json {
        let mut value = output::json::doctor(version, &dir, &datasets, &store);
        if let serde_json::Value::Object(ref mut map) = value {
            map.insert("provenance".into(), provenance_json(&dir, &datasets));
        }
        println!("{value}");
    } else {
        println!("Scribe {version}");
        println!("data directory: {}", dir.display());
        for d in &datasets {
            let status = if d.available {
                format!("available ({} verses)", d.verses)
            } else {
                "missing".to_string()
            };
            println!("{}: {}", d.witness.title, status);
        }
        println!("search index: ready (in-memory, rebuilt from store on start)");
        if !datasets
            .iter()
            .any(|d| d.witness.id == WitnessId::Lxx && d.available)
        {
            println!("\nTip: run `scribe data install lxx` to add the Greek Septuagint.");
        }
    }
    Ok(())
}

/// `scribe data install|uninstall|status`.
pub fn run_data(action: DataAction) -> Result<()> {
    match action {
        DataAction::Install { dataset } => match dataset.as_str() {
            "kjva" => {
                let dir = data_dir()?;
                let report = import_kjva(&dir)?;
                println!(
                    "Installed {} verses of KJV Apocrypha -> {}",
                    report.verses,
                    report.store_path.display()
                );
                Ok(())
            }
            "lxx" => {
                let dir = data_dir()?;
                println!("Downloading Greek LXX (Apocrypha) from the LXXMorph corpus…");
                let report = download_lxx(&dir)?;
                println!(
                    "Installed {} Greek verses -> {}",
                    report.verses,
                    report.store_path.display()
                );
                println!(
                    "Note: this dataset is for non-commercial use per the CCAT fair-use agreement."
                );
                Ok(())
            }
            "all" => {
                run_data(DataAction::Install {
                    dataset: "kjva".to_string(),
                })?;
                run_data(DataAction::Install {
                    dataset: "lxx".to_string(),
                })
            }
            other => Err(ScribeError::UnknownDataset {
                name: other.to_string(),
            }),
        },
        DataAction::Uninstall { dataset } => {
            let witness = match dataset.as_str() {
                "kjva" => WitnessId::KjvApocrypha,
                "lxx" => WitnessId::Lxx,
                other => {
                    return Err(ScribeError::UnknownDataset {
                        name: other.to_string(),
                    })
                }
            };
            let dir = data_dir()?;
            let store_file = dir.join("store").join(witness.store_file());
            if !store_file.exists() {
                return Err(ScribeError::NotInstalled { name: dataset });
            }
            fs::remove_file(&store_file).map_err(|e| ScribeError::Io {
                path: store_file.display().to_string(),
                source: e,
            })?;
            let _ = fs::remove_file(dir.join("meta").join(format!("{}.json", dataset)));
            println!("Removed {} dataset.", witness.meta().title);
            Ok(())
        }
        DataAction::Status { json } => {
            let dir = data_dir()?;
            let store = Store::open(&dir)?;
            let datasets = store.datasets();
            if json {
                println!("{}", output::json::data_status(&datasets));
            } else {
                for d in &datasets {
                    let status = if d.available {
                        format!("installed ({} verses)", d.verses)
                    } else {
                        "not installed".to_string()
                    };
                    println!("{}: {}", d.witness.title, status);
                }
            }
            Ok(())
        }
    }
}

/// Import Greek data from a raw directory (used by tests with a fixture).
#[allow(dead_code)]
pub fn import_lxx_from_raw(raw_dir: &std::path::Path, data_dir: &std::path::Path) -> Result<()> {
    // Copy the fixture files into the data dir's raw/lxx then import.
    let target = data_dir.join("raw").join("lxx");
    fs::create_dir_all(&target).map_err(|e| ScribeError::Io {
        path: target.display().to_string(),
        source: e,
    })?;
    for entry in fs::read_dir(raw_dir).map_err(|e| ScribeError::Io {
        path: raw_dir.display().to_string(),
        source: e,
    })? {
        let entry = entry.map_err(|e| ScribeError::Io {
            path: raw_dir.display().to_string(),
            source: e,
        })?;
        let name = entry.file_name();
        let name = name.to_string_lossy().to_string();
        let dst = target.join(&name);
        if dst.exists() {
            continue;
        }
        fs::copy(entry.path(), &dst).map_err(|e| ScribeError::Io {
            path: dst.display().to_string(),
            source: e,
        })?;
    }
    import_lxx_morph(data_dir)?;
    Ok(())
}

/// Used by `doctor --json` to include provenance.
pub fn provenance_json(
    dir: &std::path::Path,
    datasets: &[crate::domain::passage::DatasetInfo],
) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    for d in datasets {
        if let Some(p) = read_provenance(dir, d.witness.id) {
            map.insert(
                d.witness.id.dataset_name().to_string(),
                serde_json::to_value(p).unwrap_or(serde_json::Value::Null),
            );
        }
    }
    serde_json::Value::Object(map)
}
