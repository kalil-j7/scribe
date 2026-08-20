//! Application commands: lookup, search, compare, books.

use crate::domain::book::{resolve_book, BookId};
use crate::domain::reference::parse_reference;
use crate::domain::search::SearchQuery;
use crate::domain::source::ScriptureSource;
use crate::domain::witness::WitnessId;
use crate::error::{Result, ScribeError};
use crate::infrastructure::store::Store;
use crate::output;
use crate::text::query_terms;

use super::data::ensure_kjva;
use super::paths::data_dir;

fn open_store() -> Result<Store> {
    let dir = data_dir()?;
    ensure_kjva(&dir)?;
    Store::open(&dir)
}

fn greek_witness_error(book: &str) -> ScribeError {
    ScribeError::GreekNotInstalled {
        book: book.to_string(),
    }
}

/// `scribe <book> <ref>` and `scribe passage <book> <ref> [--greek] [--words]`.
pub fn run_passage(reference_str: &str, greek: bool, words: bool, json: bool) -> Result<()> {
    let reference = parse_reference(reference_str)?;
    let store = open_store()?;
    let witness = if greek || words {
        WitnessId::Lxx
    } else {
        WitnessId::KjvApocrypha
    };
    if words && !store.greek_installed() {
        return Err(ScribeError::GreekDataNotInstalled);
    }
    let passage = store.passage(&reference, witness).map_err(|e| match e {
        ScribeError::GreekNotInstalled { .. } => {
            greek_witness_error(reference.book.canonical_name())
        }
        other => other,
    })?;
    if words {
        if json {
            println!("{}", output::json::passage_words(&passage));
        } else {
            output::plain::passage_words(&passage);
        }
    } else if json {
        println!("{}", output::json::passage(&passage));
    } else {
        output::plain::passage(&passage);
    }
    Ok(())
}

/// `scribe word <word>` — Greek lemma study.
pub fn run_word(word: &str, json: bool) -> Result<()> {
    let store = open_store()?;
    let study = store.word_study(word)?;
    if json {
        println!("{}", output::json::word(&study));
    } else {
        output::plain::word(&study);
    }
    Ok(())
}

/// `scribe occurrences <word> [--book B]` — every token occurrence of a lemma.
pub fn run_occurrences(word: &str, book_filter: Option<&str>, json: bool) -> Result<()> {
    let book = match book_filter {
        Some(b) => Some(
            resolve_book(b)
                .map_err(|e| ScribeError::Reference(e.into()))?
                .0,
        ),
        None => None,
    };
    let store = open_store()?;
    let list = store.lemma_occurrences(word, book)?;
    if json {
        println!("{}", output::json::occurrences(&list));
    } else {
        output::plain::occurrences(&list);
    }
    Ok(())
}

/// `scribe chapter <book> <n>`.
pub fn run_chapter(reference_str: &str, json: bool) -> Result<()> {
    let reference = parse_reference(reference_str)?;
    if !reference.is_chapter() {
        return Err(ScribeError::Other(
            "`scribe chapter` expects a whole chapter, e.g. `scribe chapter sirach 2`".into(),
        ));
    }
    let store = open_store()?;
    let chapter = store.chapter(reference.book, reference.chapter, WitnessId::KjvApocrypha)?;
    if json {
        println!("{}", output::json::chapter(&chapter));
    } else {
        output::plain::chapter(&chapter);
    }
    Ok(())
}

/// `scribe compare <book> <ref>` — English + Greek side by side.
pub fn run_compare(reference_str: &str, json: bool) -> Result<()> {
    let reference = parse_reference(reference_str)?;
    let store = open_store()?;
    let english = store.passage(&reference, WitnessId::KjvApocrypha)?;
    let greek = store
        .passage(&reference, WitnessId::Lxx)
        .map_err(|e| match e {
            ScribeError::GreekNotInstalled { .. } => {
                // Clear, truthful message: Greek data may be missing *or* the
                // Greek witness may simply not have this reference.
                if store.count_verses(WitnessId::Lxx) == 0 {
                    e
                } else {
                    ScribeError::Other(format!(
                        "no Greek verse at {reference} in the installed LXX data"
                    ))
                }
            }
            other => other,
        })?;
    if json {
        println!("{}", output::json::compare(&english, &greek));
    } else {
        output::plain::compare(&english, &greek);
    }
    Ok(())
}

/// `scribe search <query> [--book B] [--greek] [--limit N]`.
pub fn run_search(
    query: &str,
    book_filter: Option<&str>,
    greek: bool,
    limit: usize,
    json: bool,
) -> Result<()> {
    let terms = query_terms(query);
    if terms.is_empty() {
        return Err(ScribeError::Other(format!(
            "search query {query:?} contains no searchable words"
        )));
    }
    let book = match book_filter {
        Some(b) => Some(
            resolve_book(b)
                .map_err(|e| ScribeError::Reference(e.into()))?
                .0,
        ),
        None => None,
    };
    let witness = if greek {
        WitnessId::Lxx
    } else {
        WitnessId::KjvApocrypha
    };
    let store = open_store()?;
    if greek && store.count_verses(WitnessId::Lxx) == 0 {
        return Err(ScribeError::GreekNotInstalled {
            book: "the corpus".to_string(),
        });
    }
    let query = SearchQuery {
        terms,
        book,
        witness,
        limit,
    };
    let hits = store.search(&query)?;
    if json {
        println!("{}", output::json::search(&query, &hits));
    } else {
        output::plain::search(&hits);
    }
    Ok(())
}

/// `scribe books` — the Apocrypha canon with chapter counts.
pub fn run_books(json: bool) -> Result<()> {
    let store = open_store()?;
    let witness = WitnessId::KjvApocrypha;
    let mut books: Vec<(BookId, Vec<(u16, u16)>)> = Vec::new();
    for book in BookId::ALL {
        if let Ok(info) = store.book_info(book, witness) {
            let chapters: Vec<(u16, u16)> =
                info.chapters.iter().map(|(c, n)| (c.get(), *n)).collect();
            if !chapters.is_empty() {
                books.push((book, chapters));
            }
        }
    }
    if json {
        println!("{}", output::json::books(&books));
    } else {
        output::plain::books(&books);
    }
    Ok(())
}
