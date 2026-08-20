//! Plain-text output. Deliberately no ANSI/color: output is always
//! pipe-friendly, so "disable color when redirected" is trivially satisfied.

use crate::domain::book::BookId;
use crate::domain::coverage::WitnessCoverage;
use crate::domain::passage::{Chapter, Passage};
use crate::domain::search::SearchHit;
use crate::domain::witness::WitnessId;
use crate::domain::word::{LemmaResolution, Morphology, OccurrenceList, WordStudy};

const SNIPPET_MAX: usize = 200;

fn witness_label(witness: WitnessId) -> &'static str {
    match witness {
        WitnessId::KjvApocrypha => "KJV (1769)",
        WitnessId::Lxx => "GREEK (LXX — RAHLFS)",
    }
}

pub fn passage(p: &Passage) {
    let header = if p.verses.len() == 1 {
        format!(
            "{} — {}",
            p.reference.to_short().to_uppercase(),
            witness_label(p.witness)
        )
    } else {
        format!(
            "{} — {}",
            p.reference.to_display().to_uppercase(),
            witness_label(p.witness)
        )
    };
    println!("{header}");
    println!();
    if p.verses.len() == 1 {
        println!("{}", p.verses[0].text);
    } else {
        let width = p
            .verses
            .iter()
            .map(|v| v.verse.get().to_string().len())
            .max()
            .unwrap_or(2);
        for v in &p.verses {
            println!("{:>width$}  {}", v.verse.get(), v.text, width = width);
        }
    }
}

/// `scribe <ref> --words` — token-level Greek word view.
pub fn passage_words(p: &Passage) {
    let header = format!("{} — GREEK WORDS", p.reference.to_display().to_uppercase());
    println!("{header}");
    println!();

    let mut rows: Vec<(String, String, String)> = Vec::new(); // surface, lemma, morph
    let mut verse_marks: Vec<String> = Vec::new();
    for v in &p.verses {
        for (i, t) in v.tokens.iter().enumerate() {
            if i == 0 {
                verse_marks.push(format!("{}:{}", v.chapter.get(), v.verse.get()));
            } else {
                verse_marks.push(String::new());
            }
            rows.push((
                t.surface.clone(),
                t.lemma.clone().unwrap_or_default(),
                t.morph.clone().unwrap_or_default(),
            ));
        }
    }
    let w_surf = rows
        .iter()
        .map(|r| r.0.chars().count())
        .max()
        .unwrap_or(6)
        .max(7);
    let w_lem = rows
        .iter()
        .map(|r| r.1.chars().count())
        .max()
        .unwrap_or(5)
        .max(5);
    let w_morph = rows
        .iter()
        .map(|r| r.2.chars().count())
        .max()
        .unwrap_or(10)
        .max(10);
    println!(
        "{:<w_surf$}  {:<w_lem$}  {:<w_morph$}",
        "Surface",
        "Lemma",
        "Morphology",
        w_surf = w_surf,
        w_lem = w_lem,
        w_morph = w_morph
    );
    for (mark, (surf, lem, morph)) in verse_marks.iter().zip(rows.iter()) {
        if !mark.is_empty() && p.verses.len() > 1 {
            println!();
            println!("{mark}");
        }
        println!(
            "{:<w_surf$}  {:<w_lem$}  {:<w_morph$}",
            surf,
            lem,
            morph,
            w_surf = w_surf,
            w_lem = w_lem,
            w_morph = w_morph
        );
    }
}

/// `scribe word <word>` — lemma study.
pub fn word(study: &WordStudy) {
    let LemmaResolution::Found(found) = &study.resolution else {
        // Resolution outcomes other than "found" are surfaced as errors by
        // the application layer (ambiguous / not found); this is defensive.
        return;
    };
    let trans = found
        .transliteration
        .as_deref()
        .map(|t| format!("  ({t})"))
        .unwrap_or_default();
    println!("{}{}", found.lemma, trans);
    println!();
    println!("LEMMA");
    println!("{}", found.lemma);
    println!();
    if let Some(pos) = found.part_of_speech {
        println!("PART OF SPEECH");
        println!("{}", pos.label());
        println!();
    }
    println!("OCCURRENCES");
    println!("{}", found.total_occurrences);
    println!();
    println!("FORMS");
    for f in &found.forms {
        let desc = f
            .morphology
            .as_deref()
            .and_then(|m| Morphology(m.to_string()).description())
            .unwrap_or_default();
        println!("{:<16} {}", f.surface, desc);
    }
    if !found.books.is_empty() {
        println!();
        println!("BOOKS");
        let w = found
            .books
            .iter()
            .map(|b| b.book.canonical_name().chars().count())
            .max()
            .unwrap_or(8);
        for b in &found.books {
            println!("{:<w$}  {}", b.book.canonical_name(), b.count, w = w);
        }
    }
}

/// `scribe occurrences <word>` — every occurrence of a lemma, grouped by book.
pub fn occurrences(list: &OccurrenceList) {
    println!("{} — {} occurrences", list.lemma, list.total);
    if let Some(book) = list.book_filter {
        println!("book: {}", book.canonical_name());
    }
    println!();
    let mut current_book: Option<BookId> = None;
    for o in &list.occurrences {
        if current_book != Some(o.reference.book) {
            if current_book.is_some() {
                println!();
            }
            println!("{}", o.reference.book.canonical_name().to_uppercase());
            println!();
            current_book = Some(o.reference.book);
        }
        println!("{}", o.reference.to_short());
        println!("{}", o.verse_text);
        println!();
    }
}

pub fn chapter(c: &Chapter) {
    let header = format!(
        "{} {} — {}",
        c.book.canonical_name().to_uppercase(),
        c.chapter.get(),
        witness_label(c.witness)
    );
    println!("{header}");
    println!();
    let width = c
        .verses
        .iter()
        .map(|v| v.verse.get().to_string().len())
        .max()
        .unwrap_or(2);
    for v in &c.verses {
        println!("{:>width$}  {}", v.verse.get(), v.text, width = width);
    }
}

pub fn search(hits: &[SearchHit]) {
    if hits.is_empty() {
        println!("0 matches");
        return;
    }
    let label = witness_label(hits[0].witness);
    println!("{} matches in {}", hits.len(), label);
    println!();
    for h in hits {
        println!(
            "{} {}:{}",
            h.book.canonical_name(),
            h.chapter.get(),
            h.verse.get()
        );
        let text = snippet(&h.text);
        println!("  {text}");
        println!();
    }
}

pub fn compare(english: &Passage, greek: &Passage) {
    println!("{}", english.reference.to_display().to_uppercase());
    println!();
    println!("{}", witness_label(WitnessId::KjvApocrypha));
    for v in &english.verses {
        println!("{}", v.text);
    }
    println!();
    println!("{}", witness_label(WitnessId::Lxx));
    for v in &greek.verses {
        println!("{}", v.text);
    }
}

pub fn books(books: &[(BookId, Vec<(u16, u16)>)], coverage: &[WitnessCoverage]) {
    println!("KJV SCRIPTURE COVERAGE");
    let mut corpus = None;
    for (book, chapters) in books {
        if corpus != Some(book.corpus()) {
            println!();
            println!("{}", book.corpus().label().to_uppercase());
            println!();
            corpus = Some(book.corpus());
        }
        let last = chapters
            .iter()
            .filter(|(c, _)| *c != 0)
            .map(|(c, _)| *c)
            .max()
            .unwrap_or(0);
        if let Some(greek) = coverage.iter().find(|c| c.book == *book) {
            println!(
                "{:<22} KJV yes ({:>2} ch)  Greek {:<22} {}",
                book.canonical_name(),
                last,
                greek.status.label(),
                greek.note
            );
        } else {
            println!("{:<22} KJV yes ({:>2} ch)", book.canonical_name(), last);
        }
    }
}

fn snippet(text: &str) -> String {
    if text.chars().count() <= SNIPPET_MAX {
        text.to_string()
    } else {
        let head: String = text.chars().take(SNIPPET_MAX).collect();
        format!("{head}…")
    }
}
