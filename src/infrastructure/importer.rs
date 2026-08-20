//! Importers: bundle the KJV Apocrypha TSV into the store, and import the
//! Greek LXXMorph (Rahlfs) corpus downloaded on demand.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::domain::book::BookId;
use crate::domain::passage::ScriptureText;
use crate::domain::witness::WitnessId;
use crate::error::{Result, ScribeError};
use crate::infrastructure::store::{Row, TokRow};
use crate::text::normalize;

/// The bundled KJV 1769 Apocrypha text (public domain; extracted from the
/// CrossWire KJVA OSIS source — see `tools/extract_kjva_osis.py` and README).
pub const BUNDLED_KJVA_TSV: &str = include_str!("../../data/kjva.tsv");

pub const LXXMORPH_BASE: &str =
    "https://raw.githubusercontent.com/nathans/lxxmorph-unicode/master/";

/// (BookId, corpus file name) for the Greek Apocrypha books we import.
pub const LXX_FILES: &[(BookId, &str)] = &[
    (BookId::FirstEsdras, "17.1Esdras.txt"),
    (BookId::Judith, "20.Judith.txt"),
    (BookId::Tobit, "21.TobitBA.txt"),
    (BookId::FirstMaccabees, "23.1Macc.txt"),
    (BookId::SecondMaccabees, "24.2Macc.txt"),
    (BookId::WisdomOfSolomon, "33.Wisdom.txt"),
    (BookId::Sirach, "34.Sirach.txt"),
    (BookId::Baruch, "50.Baruch.txt"),
    (BookId::EpistleOfJeremy, "51.EpJer.txt"),
];

pub struct ImportReport {
    pub verses: u64,
    pub store_path: PathBuf,
}

/// Split a verse's printed text into (surface, normalized) word pairs.
pub fn tokens_with_surfaces(text: &str) -> Vec<(String, String)> {
    text.split_whitespace()
        .map(|w| {
            let n = normalize(w);
            (w.to_string(), n)
        })
        .filter(|(_, n)| !n.is_empty())
        .collect()
}

/// Import the bundled KJV Apocrypha TSV into `<data>/store/kjva.jsonl`.
pub fn import_kjva(data_dir: &Path) -> Result<ImportReport> {
    let mut rows: Vec<Row> = Vec::new();
    for (lineno, line) in BUNDLED_KJVA_TSV.lines().enumerate() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut fields = line.splitn(4, '\t');
        let (book, ch, v, text) = match (fields.next(), fields.next(), fields.next(), fields.next())
        {
            (Some(b), Some(c), Some(v), Some(t)) => (b, c, v, t),
            _ => {
                return Err(ScribeError::ImportFailed {
                    dataset: "kjva".into(),
                    detail: format!("malformed line {}", lineno + 1),
                })
            }
        };
        let book = crate::domain::book::resolve_book(book)
            .map_err(|e| ScribeError::ImportFailed {
                dataset: "kjva".into(),
                detail: format!("line {}: {e}", lineno + 1),
            })?
            .0;
        let ch: u16 = ch.parse().map_err(|_| ScribeError::ImportFailed {
            dataset: "kjva".into(),
            detail: format!("bad chapter on line {}", lineno + 1),
        })?;
        let v: u16 = v.parse().map_err(|_| ScribeError::ImportFailed {
            dataset: "kjva".into(),
            detail: format!("bad verse on line {}", lineno + 1),
        })?;
        let tok = tokens_with_surfaces(text)
            .into_iter()
            .map(|(s, n)| TokRow {
                s,
                n: Some(n),
                l: None,
                m: None,
            })
            .collect();
        rows.push(Row {
            b: book.canonical_name().to_string(),
            c: ch,
            v,
            t: text.to_string(),
            tok: Some(tok),
        });
    }
    write_store(data_dir, WitnessId::KjvApocrypha, &rows)?;
    write_provenance(
        data_dir,
        WitnessId::KjvApocrypha,
        crate::domain::passage::Provenance {
            dataset: "kjva".into(),
            source: "CrossWire Bible Society KJVA OSIS (gitlab.com/crosswire-bible-society/kjv), extracted by tools/extract_kjva_osis.py".into(),
            edition: "King James Version (Authorized Version) 1769, Apocrypha".into(),
            license: "Public domain (KJV 1769, USA). CrossWire grants a general public license to use this text for any purpose.".into(),
            redistribution: "Allowed (public domain / general public license)".into(),
            commercial_use: "Allowed".into(),
            imported_at: now_string(),
        },
    )?;
    Ok(ImportReport {
        verses: rows.len() as u64,
        store_path: store_path(data_dir, WitnessId::KjvApocrypha),
    })
}

/// Import the LXXMorph (Rahlfs) Greek Apocrypha text files into
/// `<data>/store/lxx.jsonl`.
///
/// The raw files must already be present under `<data>/raw/lxx/`.
pub fn import_lxx_morph(data_dir: &Path) -> Result<ImportReport> {
    let raw_dir = data_dir.join("raw").join("lxx");
    let mut rows: Vec<Row> = Vec::new();
    for (book, file) in LXX_FILES {
        let path = raw_dir.join(file);
        if !path.exists() {
            // A partial raw corpus is allowed (e.g. the test fixture); the
            // full `scribe data install lxx` always downloads every file.
            continue;
        }
        let content = fs::read_to_string(&path).map_err(|e| ScribeError::Io {
            path: path.display().to_string(),
            source: e,
        })?;
        parse_lxxmorph_file(*book, &content, &mut rows).map_err(|detail| {
            ScribeError::ImportFailed {
                dataset: "lxx".into(),
                detail: format!("{file}: {detail}"),
            }
        })?;
    }
    write_store(data_dir, WitnessId::Lxx, &rows)?;
    write_provenance(
        data_dir,
        WitnessId::Lxx,
        crate::domain::passage::Provenance {
            dataset: "lxx".into(),
            source: "CCAT LXXMorph corpus (ccat.sas.upenn.edu), Rahlfs' Septuaginta; Unicode conversion mirrored at github.com/nathans/lxxmorph-unicode".into(),
            edition: "Septuaginta, ed. Alfred Rahlfs (1935), Apocrypha books".into(),
            license: "CCAT fair-use agreement: free for non-commercial use; redistribution requires consent. Not bundled in the repository — downloaded on demand by `scribe data install lxx`.".into(),
            redistribution: "Restricted (CCAT non-commercial terms); kept as a per-user download".into(),
            commercial_use: "Not without written consent of the rights holders".into(),
            imported_at: now_string(),
        },
    )?;
    Ok(ImportReport {
        verses: rows.len() as u64,
        store_path: store_path(data_dir, WitnessId::Lxx),
    })
}

/// Download the LXXMorph Apocrypha files into `<data>/raw/lxx/`.
pub fn download_lxx(data_dir: &Path) -> Result<ImportReport> {
    let raw_dir = data_dir.join("raw").join("lxx");
    fs::create_dir_all(&raw_dir).map_err(|e| ScribeError::Io {
        path: raw_dir.display().to_string(),
        source: e,
    })?;
    for (_, file) in LXX_FILES {
        let url = format!("{LXXMORPH_BASE}{file}");
        let target = raw_dir.join(file);
        if target.exists() {
            continue;
        }
        let mut resp = ureq::get(&url).call().map_err(|e| ScribeError::Download {
            url: url.clone(),
            detail: e.to_string(),
        })?;
        let body = resp
            .body_mut()
            .read_to_string()
            .map_err(|e| ScribeError::Download {
                url: url.clone(),
                detail: e.to_string(),
            })?;
        fs::write(&target, body).map_err(|e| ScribeError::Io {
            path: target.display().to_string(),
            source: e,
        })?;
    }
    import_lxx_morph(data_dir)
}

/// Parse one LXXMorph file: verse markers (`Sir 2:1`, `Sir Prolog:1`,
/// `EpJer ` / `EpJer N`) followed by one `surface morph lemma` line per token.
fn parse_lxxmorph_file(
    book: BookId,
    content: &str,
    out: &mut Vec<Row>,
) -> std::result::Result<(), String> {
    let source_id = book
        .greek_source_id()
        .ok_or("book has no LXXMorph source id")?;
    let mut surface: Vec<String> = Vec::new();
    let mut toks: Vec<TokRow> = Vec::new();
    let mut pending: Option<(u16, u16)> = None; // (chapter, verse) of the verse being read

    let emit = |surface: &mut Vec<String>,
                toks: &mut Vec<TokRow>,
                chapter: u16,
                verse: u16,
                out: &mut Vec<Row>|
     -> std::result::Result<(), String> {
        if surface.is_empty() {
            return Err(format!("verse {chapter}:{verse} has no tokens"));
        }
        let text = surface.join(" ");
        out.push(Row {
            b: book.canonical_name().to_string(),
            c: chapter,
            v: verse,
            t: text,
            tok: Some(std::mem::take(toks)),
        });
        surface.clear();
        Ok(())
    };

    for (lineno, line) in content.lines().enumerate() {
        let line = line.trim_end();
        if line.is_empty() {
            continue;
        }
        if let Some(rest_raw) = line.strip_prefix(source_id) {
            if let Some((ch, v)) = pending {
                emit(&mut surface, &mut toks, ch, v, out)?;
            }
            let rest = rest_raw.trim();
            if rest.is_empty() {
                // e.g. `EpJer ` — the unnumbered opening verse of the epistle.
                pending = Some((1, 1));
            } else if let Some((ch, v)) = parse_marker(rest) {
                pending = Some((ch, v));
            } else if book == BookId::EpistleOfJeremy
                && !rest.is_empty()
                && rest.bytes().all(|b| b.is_ascii_digit())
            {
                // `EpJer N` numbers the verses *after* the unnumbered opener.
                let n: u16 = rest
                    .parse()
                    .map_err(|_| format!("line {}: bad verse marker {rest:?}", lineno + 1))?;
                pending = Some((1, n + 1));
            } else {
                return Err(format!("line {}: bad verse marker {rest:?}", lineno + 1));
            }
            continue;
        }
        // token line
        let (ch, v) = pending
            .ok_or_else(|| format!("line {}: tokens before any verse marker", lineno + 1))?;
        let mut parts = line.splitn(3, ' ');
        let (s, m, l) = match (parts.next(), parts.next(), parts.next()) {
            (Some(s), Some(m), Some(l)) => (s, m, l),
            _ => {
                return Err(format!(
                    "line {}: malformed token line {line:?}",
                    lineno + 1
                ))
            }
        };
        surface.push(s.to_string());
        toks.push(TokRow {
            s: s.to_string(),
            n: Some(normalize(s)),
            l: Some(l.to_string()),
            m: Some(m.to_string()),
        });
        let _ = (ch, v);
    }
    if let Some((ch, v)) = pending {
        emit(&mut surface, &mut toks, ch, v, out)?;
    }
    Ok(())
}

fn parse_marker(rest: &str) -> Option<(u16, u16)> {
    if let Some(n) = rest.strip_prefix("Prolog:") {
        return n.parse().ok().map(|v| (0u16, v));
    }
    let (ch, v) = rest.split_once(':')?;
    Some((ch.parse().ok()?, v.parse().ok()?))
}

fn store_path(data_dir: &Path, witness: WitnessId) -> PathBuf {
    data_dir.join("store").join(witness.store_file())
}

fn write_store(data_dir: &Path, witness: WitnessId, rows: &[Row]) -> Result<()> {
    let store_dir = data_dir.join("store");
    fs::create_dir_all(&store_dir).map_err(|e| ScribeError::Io {
        path: store_dir.display().to_string(),
        source: e,
    })?;
    let target = store_path(data_dir, witness);
    let tmp = target.with_extension("jsonl.tmp");
    let mut f = fs::File::create(&tmp).map_err(|e| ScribeError::Io {
        path: tmp.display().to_string(),
        source: e,
    })?;
    for row in rows {
        serde_json::to_writer(&mut f, row).map_err(|e| ScribeError::ImportFailed {
            dataset: witness.dataset_name().into(),
            detail: e.to_string(),
        })?;
        f.write_all(b"\n").map_err(|e| ScribeError::Io {
            path: tmp.display().to_string(),
            source: e,
        })?;
    }
    f.flush().map_err(|e| ScribeError::Io {
        path: tmp.display().to_string(),
        source: e,
    })?;
    fs::rename(&tmp, &target).map_err(|e| ScribeError::Io {
        path: target.display().to_string(),
        source: e,
    })?;
    // Also write the binary cache so subsequent startups are fast.
    let verses: Vec<ScriptureText> = rows
        .iter()
        .cloned()
        .map(|r| r.into_domain(witness))
        .collect::<Result<_>>()?;
    crate::infrastructure::cache::write(
        &data_dir.join("store").join(witness.cache_file()),
        witness,
        &verses,
    )?;
    Ok(())
}

fn write_provenance(
    data_dir: &Path,
    witness: WitnessId,
    provenance: crate::domain::passage::Provenance,
) -> Result<()> {
    let meta_dir = data_dir.join("meta");
    fs::create_dir_all(&meta_dir).map_err(|e| ScribeError::Io {
        path: meta_dir.display().to_string(),
        source: e,
    })?;
    let path = meta_dir.join(format!("{}.json", witness.dataset_name()));
    let content =
        serde_json::to_string_pretty(&provenance).map_err(|e| ScribeError::ImportFailed {
            dataset: witness.dataset_name().into(),
            detail: e.to_string(),
        })?;
    fs::write(&path, content).map_err(|e| ScribeError::Io {
        path: path.display().to_string(),
        source: e,
    })?;
    Ok(())
}

fn now_string() -> String {
    // No chrono dependency: use UTC-ish wall clock via std (local time).
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = secs / 86400;
    let (y, m, d) = civil_from_days(days as i64);
    format!("{y:04}-{m:02}-{d:02} (UTC)")
}

/// Convert days since 1970-01-01 to (year, month, day) — Howard Hinnant's
/// algorithm, no dependencies.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_tsv_has_expected_shape() {
        let verses = BUNDLED_KJVA_TSV
            .lines()
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .count();
        assert!(verses > 5000, "expected >5000 verses, got {verses}");
        assert!(BUNDLED_KJVA_TSV.contains("My son, if thou come to serve the Lord"));
    }

    #[test]
    fn parses_lxxmorph_snippet() {
        let sample = "\
Sir 2:1
τέκνον N2N-VSN--- τέκνον
εἰ C--------- εἰ
προσέρχῃ V1--PMS2S- ἔρχομαι προς
δουλεύειν V1--PAN--- δουλεύω
κυρίῳ N2--DSM--- κύριος
ἑτοίμασον VA--AAD2S- ἑτοιμάζω
τὴν RA--ASF--- ὁ
ψυχήν N1--ASF--- ψυχή
σου RP--GS---- σύ
εἰς P--------- εἰς
πειρασμόν N2--ASM--- πειρασμός
Sir 2:2
εὔθυνον VA--AAD2S- εὐθύνω
";
        let mut rows = Vec::new();
        parse_lxxmorph_file(BookId::Sirach, sample, &mut rows).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].b, "Sirach");
        assert_eq!((rows[0].c, rows[0].v), (2, 1));
        assert_eq!(
            rows[0].t,
            "τέκνον εἰ προσέρχῃ δουλεύειν κυρίῳ ἑτοίμασον τὴν ψυχήν σου εἰς πειρασμόν"
        );
        let toks = rows[0].tok.as_ref().unwrap();
        assert_eq!(toks.len(), 11);
        assert_eq!(toks[2].l.as_deref(), Some("ἔρχομαι προς"));
        assert_eq!(toks[2].m.as_deref(), Some("V1--PMS2S-"));
        assert_eq!(rows[1].v, 2);
    }

    #[test]
    fn parses_epjer_markers() {
        let sample = "\
EpJer 
ἀντίγραφον N2N-NSN--- ἀντίγραφον
EpJer 1
δεύτερον D--------- δεύτερον
EpJer 2
τρίτον D--------- τρίτον
";
        let mut rows = Vec::new();
        parse_lxxmorph_file(BookId::EpistleOfJeremy, sample, &mut rows).unwrap();
        assert_eq!(rows.len(), 3);
        assert_eq!((rows[0].c, rows[0].v), (1, 1));
        assert_eq!((rows[1].c, rows[1].v), (1, 2));
        assert_eq!((rows[2].c, rows[2].v), (1, 3));
    }

    #[test]
    fn parses_prologue_markers() {
        let sample = "Sir Prolog:1\nπολλῶν A1--GPN--- πολύς\nSir 1:1\nπᾶσα A1S-NSF--- πᾶς\n";
        let mut rows = Vec::new();
        parse_lxxmorph_file(BookId::Sirach, sample, &mut rows).unwrap();
        assert_eq!((rows[0].c, rows[0].v), (0, 1));
        assert_eq!((rows[1].c, rows[1].v), (1, 1));
    }
}
