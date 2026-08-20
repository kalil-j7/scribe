//! JSON output mode (supports future TUI/mobile integrations).

use serde_json::{json, Value};

use crate::domain::book::BookId;
use crate::domain::passage::{Chapter, DatasetInfo, Passage};
use crate::domain::search::{SearchHit, SearchQuery};
use crate::domain::witness::WitnessId;
use crate::domain::word::{LemmaResolution, OccurrenceList, WordStudy};
use crate::infrastructure::store::Store;

fn witness_key(w: WitnessId) -> &'static str {
    match w {
        WitnessId::KjvApocrypha => "kjv_apocrypha",
        WitnessId::Lxx => "lxx",
    }
}

fn token_value(t: &crate::domain::passage::Token) -> Value {
    json!({
        "surface": t.surface,
        "normalized": t.normalized,
        "lemma": t.lemma,
        "morph": t.morph,
    })
}

fn verse_value(v: &crate::domain::passage::ScriptureText) -> Value {
    json!({
        "chapter": v.chapter.get(),
        "verse": v.verse.get(),
        "text": v.text,
        "tokens": v.tokens.iter().map(token_value).collect::<Vec<_>>(),
    })
}

pub fn passage(p: &Passage) -> Value {
    json!({
        "kind": "passage",
        "witness": witness_key(p.witness),
        "book": p.reference.book.canonical_name(),
        "chapter": p.reference.chapter.get(),
        "start_verse": p.reference.start_verse.get(),
        "end_verse": if p.reference.is_chapter() {
            Value::Null
        } else {
            Value::from(p.reference.end_verse.get())
        },
        "reference": p.reference.to_display(),
        "verses": p.verses.iter().map(verse_value).collect::<Vec<_>>(),
    })
}

pub fn chapter(c: &Chapter) -> Value {
    json!({
        "kind": "chapter",
        "witness": witness_key(c.witness),
        "book": c.book.canonical_name(),
        "chapter": c.chapter.get(),
        "verses": c.verses.iter().map(verse_value).collect::<Vec<_>>(),
    })
}

/// `scribe <ref> --words` — the passage JSON (tokens already included) marked
/// as a word view.
pub fn passage_words(p: &Passage) -> Value {
    let mut v = passage(p);
    if let Value::Object(ref mut map) = v {
        map.insert("view".to_string(), Value::String("words".to_string()));
    }
    v
}

/// `scribe word <word>` — structured lemma study.
pub fn word(study: &WordStudy) -> Value {
    let mut base = json!({
        "kind": "word",
        "query": study.query,
        "normalized_query": study.normalized_query,
        "corpus": study.corpus.label(),
        "resolution": "found",
    });
    if let LemmaResolution::Found(found) = &study.resolution {
        if let Value::Object(ref mut map) = base {
            map.insert("lemma".into(), Value::String(found.lemma.clone()));
            map.insert("transliteration".into(), json!(found.transliteration));
            map.insert(
                "part_of_speech".into(),
                json!(found.part_of_speech.map(|p| p.label())),
            );
            map.insert(
                "occurrence_count".into(),
                Value::from(found.total_occurrences),
            );
            map.insert(
                "forms".into(),
                Value::Array(
                    found
                        .forms
                        .iter()
                        .map(|f| {
                            json!({
                                "surface": f.surface,
                                "normalized": f.normalized,
                                "morphology": f.morphology,
                                "count": f.count,
                            })
                        })
                        .collect(),
                ),
            );
            map.insert(
                "books".into(),
                Value::Array(
                    found
                        .books
                        .iter()
                        .map(|b| {
                            json!({
                                "book": b.book.canonical_name(),
                                "count": b.count,
                            })
                        })
                        .collect(),
                ),
            );
        }
    }
    base
}

/// `scribe occurrences <word>` — structured occurrence list.
pub fn occurrences(list: &OccurrenceList) -> Value {
    json!({
        "kind": "occurrences",
        "query": list.query,
        "normalized_query": list.normalized_query,
        "lemma": list.lemma,
        "corpus": list.corpus.label(),
        "book": list.book_filter.map(|b| b.canonical_name()),
        "total": list.total,
        "occurrences": list.occurrences.iter().map(|o| json!({
            "book": o.reference.book.canonical_name(),
            "chapter": o.reference.chapter.get(),
            "verse": o.reference.start_verse.get(),
            "surface": o.token_surface,
            "lemma": o.lemma,
            "morphology": o.morphology,
            "position": o.position,
            "text": o.verse_text,
        })).collect::<Vec<_>>(),
    })
}

pub fn search(query: &SearchQuery, hits: &[SearchHit]) -> Value {
    json!({
        "kind": "search",
        "witness": witness_key(query.witness),
        "query": query.terms,
        "book": query.book.map(|b| b.canonical_name()),
        "matches": hits.len(),
        "hits": hits.iter().map(|h| json!({
            "book": h.book.canonical_name(),
            "chapter": h.chapter.get(),
            "verse": h.verse.get(),
            "text": h.text,
            "score": h.score,
        })).collect::<Vec<_>>(),
    })
}

pub fn compare(english: &Passage, greek: &Passage) -> Value {
    json!({
        "kind": "compare",
        "reference": english.reference.to_display(),
        "witnesses": [
            {
                "witness": witness_key(english.witness),
                "title": english.witness.meta().title,
                "verses": english.verses.iter().map(verse_value).collect::<Vec<_>>(),
            },
            {
                "witness": witness_key(greek.witness),
                "title": greek.witness.meta().title,
                "verses": greek.verses.iter().map(verse_value).collect::<Vec<_>>(),
            }
        ],
    })
}

pub fn books(books: &[(BookId, Vec<(u16, u16)>)]) -> Value {
    json!({
        "kind": "books",
        "books": books.iter().map(|(book, chapters)| json!({
            "book": book.canonical_name(),
            "aliases": book.aliases(),
            "chapters": chapters.iter().map(|(c, n)| json!({
                "chapter": c,
                "verses": n,
            })).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
    })
}

pub fn doctor(
    version: &str,
    data_dir: &std::path::Path,
    datasets: &[DatasetInfo],
    _store: &Store,
) -> Value {
    json!({
        "kind": "doctor",
        "version": version,
        "data_dir": data_dir.display().to_string(),
        "datasets": datasets.iter().map(|d| json!({
            "id": d.witness.id.dataset_name(),
            "title": d.witness.title,
            "language": d.witness.language.label(),
            "available": d.available,
            "verses": d.verses,
            "path": d.path,
        })).collect::<Vec<_>>(),
        "search_index": "in-memory",
    })
}

pub fn data_status(datasets: &[DatasetInfo]) -> Value {
    json!({
        "kind": "data_status",
        "datasets": datasets.iter().map(|d| json!({
            "id": d.witness.id.dataset_name(),
            "title": d.witness.title,
            "available": d.available,
            "verses": d.verses,
        })).collect::<Vec<_>>(),
    })
}
