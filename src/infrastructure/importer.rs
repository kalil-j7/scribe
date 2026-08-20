//! Importers: bundle the complete KJV TSV into the store, and import the
//! Greek LXXMorph (Rahlfs) corpus downloaded on demand.

use std::fs;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use crate::domain::book::BookId;
use crate::domain::passage::ScriptureText;
use crate::domain::witness::WitnessId;
use crate::error::{Result, ScribeError};
use crate::infrastructure::store::{Row, TokRow};
use crate::text::normalize;

/// The bundled complete KJV 1769 text (public domain; extracted from the
/// CrossWire KJVA OSIS source — see `tools/extract_kjva_osis.py` and README).
pub const BUNDLED_KJVA_TSV: &str = include_str!("../../data/kjva.tsv");

pub const LXXMORPH_BASE: &str =
    "https://raw.githubusercontent.com/nathans/lxxmorph-unicode/master/";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MappingKind {
    /// Source and Scribe use the same chapter:verse grid. Sirach's prologue
    /// is deliberately excluded because KJV Apocrypha has no matching grid.
    Direct,
    /// Theodotion's standalone one-chapter files use bare verse markers.
    OneChapter,
    /// Daniel Theodotion 3:24–91 is KJV Prayer of Azariah 1:1–68.
    DanielThree,
    /// Odes 12:1–15 is one long KJV Prayer of Manasses verse.
    OdesManasses,
}

#[derive(Clone, Copy, Debug)]
pub struct LxxFile {
    book: BookId,
    file: &'static str,
    source_id: &'static str,
    mapping: MappingKind,
}

/// Explicit source-layout adapters for the selected Rahlfs/CCAT witness.
/// We intentionally do *not* include CCAT `18.2Esdras.txt` (Greek
/// Ezra-Nehemiah) or `19.Esther.txt` (lettered Esther additions): neither can
/// be truthfully addressed on the KJV grid without a dedicated crosswalk.
pub const LXX_FILES: &[LxxFile] = &[
    LxxFile {
        book: BookId::FirstEsdras,
        file: "17.1Esdras.txt",
        source_id: "1Esdr",
        mapping: MappingKind::Direct,
    },
    LxxFile {
        book: BookId::Judith,
        file: "20.Judith.txt",
        source_id: "Jdt",
        mapping: MappingKind::Direct,
    },
    LxxFile {
        book: BookId::Tobit,
        file: "21.TobitBA.txt",
        source_id: "TobBA",
        mapping: MappingKind::Direct,
    },
    LxxFile {
        book: BookId::FirstMaccabees,
        file: "23.1Macc.txt",
        source_id: "1Mac",
        mapping: MappingKind::Direct,
    },
    LxxFile {
        book: BookId::SecondMaccabees,
        file: "24.2Macc.txt",
        source_id: "2Mac",
        mapping: MappingKind::Direct,
    },
    LxxFile {
        book: BookId::WisdomOfSolomon,
        file: "33.Wisdom.txt",
        source_id: "Wis",
        mapping: MappingKind::Direct,
    },
    LxxFile {
        book: BookId::Sirach,
        file: "34.Sirach.txt",
        source_id: "Sir",
        mapping: MappingKind::Direct,
    },
    LxxFile {
        book: BookId::Baruch,
        file: "50.Baruch.txt",
        source_id: "Bar",
        mapping: MappingKind::Direct,
    },
    LxxFile {
        book: BookId::EpistleOfJeremy,
        file: "51.EpJer.txt",
        source_id: "EpJer",
        mapping: MappingKind::Direct,
    },
    LxxFile {
        book: BookId::PrayerOfAzariah,
        file: "57.DanielTh.txt",
        source_id: "DanTh",
        mapping: MappingKind::DanielThree,
    },
    LxxFile {
        book: BookId::BelAndTheDragon,
        file: "55.BelTh.txt",
        source_id: "BelTh",
        mapping: MappingKind::OneChapter,
    },
    LxxFile {
        book: BookId::Susanna,
        file: "59.SusTh.txt",
        source_id: "SusTh",
        mapping: MappingKind::OneChapter,
    },
    LxxFile {
        book: BookId::PrayerOfManasses,
        file: "28.Odes.txt",
        source_id: "Od",
        mapping: MappingKind::OdesManasses,
    },
];

pub struct ImportReport {
    pub verses: u64,
    pub skipped_source_rows: u64,
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

/// Import the bundled complete KJV + Apocrypha TSV into `<data>/store/kjva.jsonl`.
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
        let book = BookId::from_canonical_name(book).ok_or_else(|| ScribeError::ImportFailed {
            dataset: "kjva".into(),
            detail: format!("line {}: unknown canonical book {book:?}", lineno + 1),
        })?;
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
            src: None,
        });
    }
    validate_kjv_rows(&rows)?;
    write_store(data_dir, WitnessId::KjvApocrypha, &rows)?;
    write_provenance(
        data_dir,
        WitnessId::KjvApocrypha,
        crate::domain::passage::Provenance {
            dataset: "kjva".into(),
            source: "CrossWire Bible Society KJVA OSIS (gitlab.com/crosswire-bible-society/kjv), extracted by tools/extract_kjva_osis.py".into(),
            edition: "King James Version (Authorized Version) 1769, Old Testament, Apocrypha, and New Testament".into(),
            license: "Public domain (KJV 1769, USA). CrossWire grants a general public license to use this text for any purpose.".into(),
            redistribution: "Allowed (public domain / general public license)".into(),
            commercial_use: "Allowed".into(),
            imported_at: now_string(),
        },
    )?;
    Ok(ImportReport {
        verses: rows.len() as u64,
        skipped_source_rows: 0,
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
    for mapping in LXX_FILES {
        let path = raw_dir.join(mapping.file);
        if !path.exists() {
            // A partial raw corpus is allowed (e.g. the test fixture); the
            // full `scribe data install lxx` always downloads every file.
            continue;
        }
        let content = fs::read_to_string(&path).map_err(|e| ScribeError::Io {
            path: path.display().to_string(),
            source: e,
        })?;
        parse_lxxmorph_file(*mapping, &content, &mut rows).map_err(|detail| {
            ScribeError::ImportFailed {
                dataset: "lxx".into(),
                detail: format!("{}: {detail}", mapping.file),
            }
        })?;
    }
    let skipped_source_rows = validate_lxx_rows(&mut rows)?;
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
        skipped_source_rows,
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
    for mapping in LXX_FILES {
        let url = format!("{LXXMORPH_BASE}{}", mapping.file);
        let target = raw_dir.join(mapping.file);
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
    mapping: LxxFile,
    content: &str,
    out: &mut Vec<Row>,
) -> std::result::Result<(), String> {
    let mut surface: Vec<String> = Vec::new();
    let mut toks: Vec<TokRow> = Vec::new();
    let mut pending: Option<(u16, u16)> = None; // (chapter, verse) of the verse being read

    let emit = |surface: &mut Vec<String>,
                toks: &mut Vec<TokRow>,
                source_chapter: u16,
                source_verse: u16,
                out: &mut Vec<Row>|
     -> std::result::Result<(), String> {
        if surface.is_empty() {
            return Err(format!(
                "source verse {source_chapter}:{source_verse} has no tokens"
            ));
        }
        let text = surface.join(" ");
        let source_reference = if mapping.mapping == MappingKind::OneChapter {
            format!("{} {source_verse}", mapping.source_id)
        } else {
            format!("{} {source_chapter}:{source_verse}", mapping.source_id)
        };
        let target = match mapping.mapping {
            MappingKind::Direct if source_chapter > 0 => Some((source_chapter, source_verse)),
            MappingKind::Direct => None, // Sirach prologue: no KJV counterpart.
            MappingKind::OneChapter => Some((1, source_verse)),
            MappingKind::DanielThree
                if source_chapter == 3 && (24..=91).contains(&source_verse) =>
            {
                Some((1, source_verse - 23))
            }
            MappingKind::DanielThree => None,
            MappingKind::OdesManasses
                if source_chapter == 12 && (1..=15).contains(&source_verse) =>
            {
                Some((1, 1))
            }
            MappingKind::OdesManasses => None,
        };
        let Some((chapter, verse)) = target else {
            surface.clear();
            toks.clear();
            return Ok(());
        };
        if mapping.mapping == MappingKind::OdesManasses {
            if let Some(last) = out
                .last_mut()
                .filter(|r| r.b == mapping.book.canonical_name())
            {
                last.t.push(' ');
                last.t.push_str(&text);
                last.tok.get_or_insert_with(Vec::new).append(toks);
                last.src.as_mut().expect("source reference").push_str(", ");
                last.src
                    .as_mut()
                    .expect("source reference")
                    .push_str(&source_reference);
                surface.clear();
                return Ok(());
            }
        }
        out.push(Row {
            b: mapping.book.canonical_name().to_string(),
            c: chapter,
            v: verse,
            t: text,
            tok: Some(std::mem::take(toks)),
            src: Some(source_reference),
        });
        surface.clear();
        Ok(())
    };

    for (lineno, line) in content.lines().enumerate() {
        let line = line.trim_end();
        if line.is_empty() {
            continue;
        }
        if let Some(rest_raw) = line.strip_prefix(mapping.source_id) {
            if let Some((ch, v)) = pending {
                emit(&mut surface, &mut toks, ch, v, out)?;
            }
            let rest = rest_raw.trim();
            if mapping.mapping == MappingKind::OdesManasses && !rest.starts_with("12:") {
                pending = None;
                continue;
            }
            if rest.is_empty() && mapping.book == BookId::EpistleOfJeremy {
                // e.g. `EpJer ` — the unnumbered opening verse of the epistle.
                pending = Some((1, 1));
            } else if rest.is_empty() && mapping.mapping == MappingKind::OdesManasses {
                // Bare `Od` markers divide the Odes. They are not verses.
                pending = None;
            } else if let Some((ch, v)) = parse_marker(rest) {
                pending = Some((ch, v));
            } else if mapping.book == BookId::EpistleOfJeremy
                && !rest.is_empty()
                && rest.bytes().all(|b| b.is_ascii_digit())
            {
                // `EpJer N` numbers the verses *after* the unnumbered opener.
                let n: u16 = rest
                    .parse()
                    .map_err(|_| format!("line {}: bad verse marker {rest:?}", lineno + 1))?;
                pending = Some((1, n + 1));
            } else if mapping.mapping == MappingKind::OneChapter
                && rest.bytes().all(|b| b.is_ascii_digit())
            {
                let v: u16 = rest
                    .parse()
                    .map_err(|_| format!("line {}: bad verse marker {rest:?}", lineno + 1))?;
                pending = Some((1, v));
            } else {
                return Err(format!("line {}: bad verse marker {rest:?}", lineno + 1));
            }
            continue;
        }
        // token line
        let Some((ch, v)) = pending else {
            if mapping.mapping == MappingKind::OdesManasses {
                // Odes headings are unnumbered and intentionally outside this
                // adapter's one supported Ode.
                continue;
            }
            return Err(format!(
                "line {}: tokens before any verse marker",
                lineno + 1
            ));
        };
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

/// Sort normalized target rows and reject mappings that would make a Greek
/// reference ambiguous or fall outside the bundled KJV grid.
fn validate_lxx_rows(rows: &mut Vec<Row>) -> Result<u64> {
    use crate::domain::coverage::{lxx_coverage, CoverageStatus};
    let mut kjv_refs = std::collections::HashSet::new();
    for line in BUNDLED_KJVA_TSV
        .lines()
        .filter(|line| !line.starts_with('#') && !line.is_empty())
    {
        let mut fields = line.splitn(4, '\t');
        let (Some(book), Some(chapter), Some(verse), _) =
            (fields.next(), fields.next(), fields.next(), fields.next())
        else {
            continue;
        };
        let book = BookId::from_canonical_name(book).ok_or_else(|| ScribeError::ImportFailed {
            dataset: "lxx".into(),
            detail: format!("unknown canonical book {book:?}"),
        })?;
        let chapter = chapter
            .parse::<u16>()
            .map_err(|_| ScribeError::ImportFailed {
                dataset: "lxx".into(),
                detail: "bad bundled KJV chapter".into(),
            })?;
        let verse = verse
            .parse::<u16>()
            .map_err(|_| ScribeError::ImportFailed {
                dataset: "lxx".into(),
                detail: "bad bundled KJV verse".into(),
            })?;
        kjv_refs.insert((book, chapter, verse));
    }
    let mut skipped = 0u64;
    rows.retain(|row| {
        let Some(book) = BookId::from_canonical_name(&row.b) else {
            return true;
        };
        if lxx_coverage(book).expect("Greek rows are Apocrypha").status == CoverageStatus::Partial
            && !kjv_refs.contains(&(book, row.c, row.v))
        {
            skipped += 1;
            return false;
        }
        true
    });
    rows.sort_by(|a, b| (a.b.as_str(), a.c, a.v).cmp(&(b.b.as_str(), b.c, b.v)));
    let mut seen = std::collections::HashSet::new();
    for row in rows {
        let book =
            BookId::from_canonical_name(&row.b).ok_or_else(|| ScribeError::ImportFailed {
                dataset: "lxx".into(),
                detail: format!("unknown canonical book {:?}", row.b),
            })?;
        // Partial source grids remain searchable by their printed CCAT
        // references, but are never offered to `compare`; only exact/full
        // adapters may claim a KJV-target reference at import time.
        if lxx_coverage(book).expect("Greek rows are Apocrypha").status != CoverageStatus::Partial
            && !kjv_refs.contains(&(book, row.c, row.v))
        {
            return Err(ScribeError::ImportFailed {
                dataset: "lxx".into(),
                detail: format!(
                    "{} {}:{} is outside the KJV target grid (source {})",
                    row.b,
                    row.c,
                    row.v,
                    row.src.as_deref().unwrap_or("?")
                ),
            });
        }
        if !seen.insert((book, row.c, row.v)) {
            return Err(ScribeError::ImportFailed {
                dataset: "lxx".into(),
                detail: format!("duplicate Greek target {} {}:{}", row.b, row.c, row.v),
            });
        }
    }
    Ok(skipped)
}

fn validate_kjv_rows(rows: &[Row]) -> Result<()> {
    let mut seen = std::collections::HashSet::new();
    let mut books = std::collections::HashSet::new();
    for row in rows {
        let book =
            BookId::from_canonical_name(&row.b).ok_or_else(|| ScribeError::ImportFailed {
                dataset: "kjva".into(),
                detail: format!("unknown canonical book {:?}", row.b),
            })?;
        if row.c == 0 || row.v == 0 || !seen.insert((book, row.c, row.v)) {
            return Err(ScribeError::ImportFailed {
                dataset: "kjva".into(),
                detail: format!(
                    "duplicate or invalid reference {} {}:{}",
                    row.b, row.c, row.v
                ),
            });
        }
        books.insert(book);
    }
    for book in BookId::ALL {
        if !books.contains(&book) {
            return Err(ScribeError::ImportFailed {
                dataset: "kjva".into(),
                detail: format!("bundled KJV source is missing {}", book.canonical_name()),
            });
        }
    }
    Ok(())
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
    let file = fs::File::create(&tmp).map_err(|e| ScribeError::Io {
        path: tmp.display().to_string(),
        source: e,
    })?;
    let mut f = BufWriter::new(file);
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
        assert!(
            verses > 36000,
            "expected complete KJV/KJVA corpus, got {verses}"
        );
        assert!(BUNDLED_KJVA_TSV.contains("In the beginning God created the heaven and the earth."));
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
        parse_lxxmorph_file(LXX_FILES[6], sample, &mut rows).unwrap();
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
        parse_lxxmorph_file(LXX_FILES[8], sample, &mut rows).unwrap();
        assert_eq!(rows.len(), 3);
        assert_eq!((rows[0].c, rows[0].v), (1, 1));
        assert_eq!((rows[1].c, rows[1].v), (1, 2));
        assert_eq!((rows[2].c, rows[2].v), (1, 3));
    }

    #[test]
    fn parses_prologue_markers() {
        let sample = "Sir Prolog:1\nπολλῶν A1--GPN--- πολύς\nSir 1:1\nπᾶσα A1S-NSF--- πᾶς\n";
        let mut rows = Vec::new();
        parse_lxxmorph_file(LXX_FILES[6], sample, &mut rows).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!((rows[0].c, rows[0].v), (1, 1));
    }

    #[test]
    fn maps_theodotion_additions_and_odes_without_blending_recensions() {
        let azariah = "DanTh 3:24\nκαὶ C--------- καί\nDanTh 3:57\nμέσῳ N2--DSM--- μέσος\nDanTh 3:91\nΝαβουχοδονοσορ N---NSM--- Ναβουχοδονοσορ\n";
        let mut rows = Vec::new();
        parse_lxxmorph_file(LXX_FILES[9], azariah, &mut rows).unwrap();
        assert_eq!(
            rows.iter().map(|r| (r.c, r.v)).collect::<Vec<_>>(),
            vec![(1, 1), (1, 34), (1, 68)]
        );
        assert_eq!(rows[0].src.as_deref(), Some("DanTh 3:24"));

        let susanna = "SusTh 1\nκαὶ C--------- καί\nSusTh 32\nδίκαιος A3--NSM--- δίκαιος\nSusTh 64\nἀμήν D--------- ἀμήν\n";
        rows.clear();
        parse_lxxmorph_file(LXX_FILES[11], susanna, &mut rows).unwrap();
        assert_eq!(
            rows.iter().map(|r| r.v).collect::<Vec<_>>(),
            vec![1, 32, 64]
        );

        let bel = "BelTh 1\nκαὶ C--------- καί\nBelTh 21\nθεός N2--NSM--- θεός\nBelTh 42\nἀμήν D--------- ἀμήν\n";
        rows.clear();
        parse_lxxmorph_file(LXX_FILES[10], bel, &mut rows).unwrap();
        assert_eq!(
            rows.iter().map(|r| r.v).collect::<Vec<_>>(),
            vec![1, 21, 42]
        );

        let manasses = "Od 12:1\nκύριε N2--VSM--- κύριος\nOd 12:8\nἁμάρτηκα VAI-AAI1S- ἁμαρτάνω\nOd 12:15\nἀμήν D--------- ἀμήν\n";
        rows.clear();
        parse_lxxmorph_file(LXX_FILES[12], manasses, &mut rows).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!((rows[0].c, rows[0].v), (1, 1));
        assert!(rows[0].src.as_deref().unwrap().contains("Od 12:1"));
        assert!(rows[0].src.as_deref().unwrap().contains("Od 12:15"));
    }
}
