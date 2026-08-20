//! Integration tests: exercise the real data pipeline (import -> store ->
//! lookup/search) and the actual CLI binary against a temporary data dir.
//!
//! The KJV Apocrypha dataset is the bundled public-domain text; the Greek
//! fixture is a small excerpt of the CCAT LXXMorph corpus (non-commercial
//! fair-use, see tests/fixtures/lxx/README.md).

use std::path::{Path, PathBuf};
use std::process::Command;

use scribe::domain::reference::parse_reference;
use scribe::domain::source::ScriptureSource;
use scribe::domain::witness::WitnessId;
use scribe::error::ScribeError;
use scribe::infrastructure::importer::import_kjva;
use scribe::infrastructure::store::Store;

fn fresh_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("scribe-it-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn open_english_store(dir: &Path) -> Store {
    import_kjva(dir).expect("bundled kjva import");
    Store::open(dir).expect("store open")
}

#[test]
fn english_pipeline_known_passage() {
    let dir = fresh_dir("eng");
    let store = open_english_store(&dir);

    let r = parse_reference("Sirach 2:1").unwrap();
    let passage = store.passage(&r, WitnessId::KjvApocrypha).unwrap();
    assert_eq!(passage.verses.len(), 1);
    assert_eq!(
        passage.verses[0].text,
        "My son, if thou come to serve the Lord, prepare thy soul for temptation."
    );
}

#[test]
fn english_chapter_lookup() {
    let dir = fresh_dir("chap");
    let store = open_english_store(&dir);
    let chapter = store
        .chapter(
            scribe::domain::book::BookId::Sirach,
            scribe::domain::reference::ChapterNumber::new(2),
            WitnessId::KjvApocrypha,
        )
        .unwrap();
    assert_eq!(chapter.verses.len(), 18);
    assert_eq!(chapter.verses[0].verse.get(), 1);
    assert_eq!(chapter.verses[17].verse.get(), 18);
}

#[test]
fn english_search_finds_wisdom_in_sirach() {
    let dir = fresh_dir("search");
    let store = open_english_store(&dir);
    let hits = store
        .search(&scribe::domain::search::SearchQuery {
            terms: vec!["wisdom".to_string()],
            book: Some(scribe::domain::book::BookId::Sirach),
            witness: WitnessId::KjvApocrypha,
            limit: 50,
        })
        .unwrap();
    assert!(!hits.is_empty());
    assert!(hits
        .iter()
        .all(|h| h.book == scribe::domain::book::BookId::Sirach));
    // The best-scoring verse has the phrase twice.
    assert!(hits[0].score >= 2);
}

#[test]
fn out_of_range_errors_are_descriptive() {
    let dir = fresh_dir("range");
    let store = open_english_store(&dir);
    let r = parse_reference("Sirach 90").unwrap();
    let err = store.passage(&r, WitnessId::KjvApocrypha).unwrap_err();
    match err {
        ScribeError::ChapterOutOfRange { max, .. } => assert_eq!(max, 51),
        other => panic!("expected ChapterOutOfRange, got {other}"),
    }
    let r = parse_reference("Sirach 2:99").unwrap();
    match store.passage(&r, WitnessId::KjvApocrypha).unwrap_err() {
        ScribeError::VerseOutOfRange { max, .. } => assert_eq!(max, 18),
        other => panic!("expected VerseOutOfRange, got {other}"),
    }
}

#[test]
fn alias_resolution_through_parser() {
    for input in ["Sirach 2:1", "Ecclesiasticus 2:1", "Ecclus 2:1", "sir 2:1"] {
        assert_eq!(
            parse_reference(input).unwrap().book,
            scribe::domain::book::BookId::Sirach,
            "{input}"
        );
    }
    assert_eq!(
        parse_reference("1 Maccabees 3:1").unwrap().book,
        scribe::domain::book::BookId::FirstMaccabees
    );
}

#[test]
fn greek_pipeline_known_passage() {
    let dir = fresh_dir("grk");
    let raw = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/lxx");
    scribe::application::data::import_lxx_from_raw(&raw, &dir).unwrap();
    let store = Store::open(&dir).unwrap();
    let r = parse_reference("Sirach 2:1").unwrap();
    let passage = store.passage(&r, WitnessId::Lxx).unwrap();
    assert_eq!(
        passage.verses[0].text,
        "τέκνον εἰ προσέρχῃ δουλεύειν κυρίῳ ἑτοίμασον τὴν ψυχήν σου εἰς πειρασμόν"
    );
    // tokens carry lemma + morphology from the corpus
    let tokens = &passage.verses[0].tokens;
    assert_eq!(tokens[0].lemma.as_deref(), Some("τέκνον"));
    assert_eq!(tokens[2].morph.as_deref(), Some("V1--PMS2S-"));
    // prologue stored at chapter 0
    let pro = store
        .passage(&parse_reference("Sirach 0:1").unwrap(), WitnessId::Lxx)
        .unwrap();
    assert!(pro.verses[0].text.starts_with("πολλῶν"));
}

#[test]
fn cli_binary_serves_the_required_commands() {
    let dir = fresh_dir("cli");
    let bin = env!("CARGO_BIN_EXE_scribe");

    let run = |args: &[&str]| {
        Command::new(bin)
            .args(args)
            .env("SCRIBE_DATA_DIR", &dir)
            .output()
            .expect("spawn scribe")
    };

    let out = run(&["sirach", "2:1"]);
    assert!(
        out.status.success(),
        "sirach 2:1 failed: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("My son, if thou come to serve the Lord"));
    assert!(stdout.contains("SIRACH 2:1 — KJV APOCRYPHA"));

    let out = run(&["sirach", "2"]);
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("Set thy heart aright"));

    let out = run(&["search", "wisdom", "--book", "sirach"]);
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("matches"));

    let out = run(&["sirach", "90"]);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("Sirach has no chapter 90"));

    let out = run(&["doctor"]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("KJV Apocrypha: available (5705 verses)"));
    assert!(stdout.contains("Greek (LXX): missing"));

    let out = run(&["compare", "sirach", "2:1"]);
    // Greek not installed in this fresh dir: the error must say so.
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("data install lxx"));
}

fn install_greek_fixture(dir: &Path) {
    let raw = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/lxx");
    scribe::application::data::import_lxx_from_raw(&raw, dir).unwrap();
}

fn greek_cli(dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_scribe"))
        .args(args)
        .env("SCRIBE_DATA_DIR", dir)
        .output()
        .expect("spawn scribe")
}

#[test]
fn word_command_works_on_real_greek_fixture() {
    let dir = fresh_dir("word");
    install_greek_fixture(&dir);

    let out = greek_cli(&dir, &["word", "πειρασμός"]);
    assert!(
        out.status.success(),
        "word πειρασμός failed: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("πειρασμός"));
    assert!(stdout.contains("OCCURRENCES"));
    assert!(stdout.contains("Sirach"));
}

#[test]
fn word_inflected_form_resolves_to_same_lemma() {
    let dir = fresh_dir("wordinf");
    install_greek_fixture(&dir);

    let out = greek_cli(&dir, &["word", "πειρασμόν"]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("πειρασμός"),
        "inflected form must resolve to the lemma:\n{stdout}"
    );
    assert!(
        stdout.contains("πειρασμόν"),
        "surface form must be listed:\n{stdout}"
    );
}

#[test]
fn occurrences_returns_verified_sirach_2_1() {
    let dir = fresh_dir("occ");
    install_greek_fixture(&dir);

    let out = greek_cli(&dir, &["occurrences", "πειρασμός"]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Sirach 2:1"),
        "expected Sirach 2:1 in:\n{stdout}"
    );
    assert!(stdout.contains("πειρασμόν"));
}

#[test]
fn occurrences_book_filter_accepts_aliases() {
    let dir = fresh_dir("occbook");
    install_greek_fixture(&dir);

    for alias in ["sirach", "ecclesiasticus", "ecclus"] {
        let out = greek_cli(&dir, &["occurrences", "πειρασμός", "--book", alias]);
        assert!(
            out.status.success(),
            "--book {alias} failed: {:?}",
            String::from_utf8_lossy(&out.stderr)
        );
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(stdout.contains("Sirach 2:1"));
        assert!(stdout.contains("1 occurrences"));
    }
}

#[test]
fn words_view_shows_surface_and_lemma() {
    let dir = fresh_dir("words");
    install_greek_fixture(&dir);

    let out = greek_cli(&dir, &["sirach", "2:1", "--words"]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("πειρασμόν"),
        "surface form missing:\n{stdout}"
    );
    assert!(stdout.contains("πειρασμός"), "lemma missing:\n{stdout}");
    assert!(stdout.contains("Morphology"));
}

#[test]
fn word_json_is_structured() {
    let dir = fresh_dir("wordjson");
    install_greek_fixture(&dir);

    let out = greek_cli(&dir, &["word", "πειρασμόν", "--json"]);
    assert!(out.status.success());
    let value: serde_json::Value = serde_json::from_slice(&out.stdout).expect("valid JSON");
    assert_eq!(value["kind"], "word");
    assert_eq!(value["resolution"], "found");
    assert_eq!(value["lemma"], "πειρασμός");
    assert_eq!(value["query"], "πειρασμόν");
    assert!(value["occurrence_count"].is_number());
    assert!(value["forms"].is_array());
    assert!(value["books"].is_array());
}

#[test]
fn occurrences_json_is_structured() {
    let dir = fresh_dir("occjson");
    install_greek_fixture(&dir);

    let out = greek_cli(&dir, &["occurrences", "πειρασμός", "--json"]);
    assert!(out.status.success());
    let value: serde_json::Value = serde_json::from_slice(&out.stdout).expect("valid JSON");
    assert_eq!(value["kind"], "occurrences");
    assert_eq!(value["lemma"], "πειρασμός");
    assert_eq!(value["total"], 1);
    let occ = &value["occurrences"][0];
    assert_eq!(occ["book"], "Sirach");
    assert_eq!(occ["verse"], 1);
    assert_eq!(occ["surface"], "πειρασμόν");
    assert_eq!(occ["lemma"], "πειρασμός");
    assert!(occ["position"].is_number());
}

#[test]
fn word_requires_greek_data() {
    let dir = fresh_dir("wordmissing");
    let out = greek_cli(&dir, &["word", "πειρασμός"]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("Greek data is not installed"), "{stderr}");
    assert!(stderr.contains("scribe data install lxx"), "{stderr}");

    let out = greek_cli(&dir, &["sirach", "2:1", "--words"]);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("Greek data is not installed"));
}

#[test]
fn word_not_found_has_useful_error() {
    let dir = fresh_dir("wordnf");
    install_greek_fixture(&dir);
    let out = greek_cli(&dir, &["word", "foobar"]);
    assert!(!out.status.success());
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("no Greek word or lemma matching \"foobar\"")
    );
}
