//! Greek word-study domain: lemmas, forms, occurrences, corpora.
//!
//! The distinction the whole module protects:
//! `surface form != normalized form != lemma`.
//! An [`Occurrence`] is a real domain object (reference + token + lemma +
//! morphology + position), never a bare `String`.

use serde::{Deserialize, Serialize};

use super::book::BookId;
use super::passage::Token;
use super::reference::PassageRef;
use super::witness::WitnessId;

/// Which corpus a witness belongs to. Only `Apocrypha` has data today; the
/// other variants are the joints for future OT/NT corpora.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Corpus {
    Apocrypha,
    SeptuagintOt,
    NewTestament,
}

impl Corpus {
    pub fn label(self) -> &'static str {
        match self {
            Corpus::Apocrypha => "apocrypha",
            Corpus::SeptuagintOt => "septuagint_ot",
            Corpus::NewTestament => "new_testament",
        }
    }

    /// The corpus a witness currently maps to. Every Greek witness installed
    /// today is Apocrypha; this is the single place that mapping lives.
    pub fn from_witness(w: WitnessId) -> Corpus {
        match w {
            WitnessId::KjvApocrypha | WitnessId::Lxx => Corpus::Apocrypha,
        }
    }
}

/// Part of speech derived from the morphological parse tag of the corpus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PartOfSpeech {
    Noun,
    Verb,
    Adjective,
    Adverb,
    Pronoun,
    Article,
    Conjunction,
    Preposition,
    Particle,
    Interjection,
    Indeclinable,
    Unknown,
}

impl PartOfSpeech {
    pub fn label(self) -> &'static str {
        match self {
            PartOfSpeech::Noun => "noun",
            PartOfSpeech::Verb => "verb",
            PartOfSpeech::Adjective => "adjective",
            PartOfSpeech::Adverb => "adverb",
            PartOfSpeech::Pronoun => "pronoun",
            PartOfSpeech::Article => "article",
            PartOfSpeech::Conjunction => "conjunction",
            PartOfSpeech::Preposition => "preposition",
            PartOfSpeech::Particle => "particle",
            PartOfSpeech::Interjection => "interjection",
            PartOfSpeech::Indeclinable => "indeclinable",
            PartOfSpeech::Unknown => "unknown",
        }
    }
}

/// The morphological parse tag as it comes from the corpus (CCAT/Packard
/// coding, e.g. `N2--ASM---`), plus a human-readable decoding.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Morphology(pub String);

impl Morphology {
    /// Part of speech derived from the first letter of the tag.
    pub fn part_of_speech(&self) -> Option<PartOfSpeech> {
        let code = self.0.chars().next()?;
        let subtype = self.0.chars().nth(1).unwrap_or('-');
        Some(match code {
            'N' => PartOfSpeech::Noun,
            'V' => PartOfSpeech::Verb,
            'A' => PartOfSpeech::Adjective,
            'D' => PartOfSpeech::Adverb,
            'R' if subtype == 'A' => PartOfSpeech::Article,
            'R' => PartOfSpeech::Pronoun,
            'C' => PartOfSpeech::Conjunction,
            'P' => PartOfSpeech::Preposition,
            'X' => PartOfSpeech::Particle,
            'I' => PartOfSpeech::Interjection,
            'M' => PartOfSpeech::Indeclinable,
            _ => return None,
        })
    }

    /// A readable decoding of the CCAT/Packard parse code, e.g.
    /// `noun acc sg masc`, `verb pres act ptcp nom sg neut`.
    pub fn description(&self) -> Option<String> {
        let tag: Vec<char> = self.0.chars().collect();
        if tag.len() < 2 {
            return None;
        }
        let pos = self.part_of_speech()?;
        // Layout: variable-length type code (1-3 letters/digits) + '-'
        // + parse code (letters/digits, '-' padded to a 10-char tag).
        let type_part: String = tag.iter().take_while(|c| **c != '-').take(3).collect();
        let parse: String = tag
            .iter()
            .skip(type_part.chars().count() + 1)
            .filter(|c| **c != '-')
            .collect();
        let mut words = vec![pos.label().to_string()];
        match pos {
            PartOfSpeech::Noun | PartOfSpeech::Article | PartOfSpeech::Pronoun => {
                if let Some(rest) = case_number_gender(&parse) {
                    words.push(rest);
                }
            }
            PartOfSpeech::Adjective => {
                if let Some(rest) = case_number_gender(&parse) {
                    words.push(rest);
                }
                // 4th parse column: degree (C compar / S superl)
                match parse.chars().nth(3) {
                    Some('C') => words.push("compar".into()),
                    Some('S') => words.push("superl".into()),
                    _ => {}
                }
            }
            PartOfSpeech::Verb => {
                let parse_chars: Vec<char> = parse.chars().collect();
                let t = parse_chars.first().copied().unwrap_or('-');
                let v = parse_chars.get(1).copied().unwrap_or('-');
                let m = parse_chars.get(2).copied().unwrap_or('-');
                words.push(tense_label(t).to_string());
                words.push(voice_label(v).to_string());
                if m == 'P' {
                    // participle: remaining columns are case/number/gender
                    words.push("ptcp".to_string());
                    if let Some(rest) = case_number_gender(&parse[3..]) {
                        words.push(rest);
                    }
                } else {
                    words.push(mood_label(m).to_string());
                    let person = parse_chars.get(3).copied().unwrap_or('-');
                    let number = parse_chars.get(4).copied().unwrap_or('-');
                    if person.is_ascii_digit() {
                        words.push(person.to_string());
                    }
                    words.push(number_label(number).to_string());
                }
            }
            _ => {}
        }
        Some(words.join(" "))
    }
}

fn case_number_gender(parse: &str) -> Option<String> {
    let mut out = Vec::new();
    let case = parse.chars().next()?;
    let number = parse.chars().nth(1).unwrap_or('-');
    let gender = parse.chars().nth(2).unwrap_or('-');
    out.push(case_label(case).to_string());
    out.push(number_label(number).to_string());
    if gender != '-' {
        out.push(gender_label(gender).to_string());
    }
    Some(out.join(" "))
}

fn case_label(c: char) -> &'static str {
    match c {
        'N' => "nom",
        'G' => "gen",
        'D' => "dat",
        'A' => "acc",
        'V' => "voc",
        _ => "?",
    }
}

fn number_label(c: char) -> &'static str {
    match c {
        'S' => "sg",
        'D' => "dual",
        'P' => "pl",
        _ => "?",
    }
}

fn gender_label(c: char) -> &'static str {
    match c {
        'M' => "masc",
        'F' => "fem",
        'N' => "neut",
        _ => "?",
    }
}

fn tense_label(c: char) -> &'static str {
    match c {
        'P' => "pres",
        'I' => "imperf",
        'F' => "fut",
        'A' => "aor",
        'X' => "perf",
        'Y' => "pluperf",
        _ => "?",
    }
}

fn voice_label(c: char) -> &'static str {
    match c {
        'A' => "act",
        'M' => "mid",
        'P' => "pass",
        _ => "?",
    }
}

fn mood_label(c: char) -> &'static str {
    match c {
        'I' => "ind",
        'D' => "imper",
        'S' => "subj",
        'O' => "opt",
        'N' => "inf",
        _ => "?",
    }
}

/// One distinct surface form of a lemma, with its morphology and count.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WordForm {
    pub surface: String,
    pub normalized: String,
    pub morphology: Option<String>,
    pub count: u32,
}

/// Occurrence count of a lemma in one book.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BookCount {
    pub book: BookId,
    pub count: u32,
}

/// The full study of one resolved lemma.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LemmaStudy {
    /// Dictionary form as printed in the corpus (e.g. `πειρασμός`).
    pub lemma: String,
    /// Accent/case-insensitive key used for matching.
    pub normalized_lemma: String,
    pub transliteration: Option<String>,
    pub part_of_speech: Option<PartOfSpeech>,
    pub total_occurrences: u32,
    pub forms: Vec<WordForm>,
    pub books: Vec<BookCount>,
}

/// A lemma candidate offered when a surface form is ambiguous.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LemmaCandidate {
    pub lemma: String,
    pub normalized_lemma: String,
    pub part_of_speech: Option<PartOfSpeech>,
    pub occurrence_count: u32,
}

/// The outcome of resolving a user query to a lemma study.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LemmaResolution {
    Found(Box<LemmaStudy>),
    Ambiguous {
        surface: String,
        normalized: String,
        candidates: Vec<LemmaCandidate>,
    },
    NotFound {
        surface: String,
        normalized: String,
    },
}

/// The result of `scribe word <query>`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WordStudy {
    pub query: String,
    pub normalized_query: String,
    pub corpus: Corpus,
    pub resolution: LemmaResolution,
}

/// One token occurrence of a lemma inside a verse.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Occurrence {
    pub reference: PassageRef,
    pub witness: WitnessId,
    pub corpus: Corpus,
    pub token_surface: String,
    pub token_normalized: String,
    pub lemma: String,
    pub morphology: Option<String>,
    /// 0-based position of the token inside the verse.
    pub position: usize,
    pub verse_text: String,
}

/// The result of `scribe occurrences <query> [--book B]`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OccurrenceList {
    pub query: String,
    pub normalized_query: String,
    pub lemma: String,
    pub normalized_lemma: String,
    pub corpus: Corpus,
    pub book_filter: Option<BookId>,
    pub total: u32,
    pub occurrences: Vec<Occurrence>,
}

/// Build an [`Occurrence`] for one token of a verse.
pub fn occurrence_from_token(
    reference: PassageRef,
    witness: WitnessId,
    verse_text: &str,
    position: usize,
    token: &Token,
    lemma: &str,
) -> Occurrence {
    Occurrence {
        reference,
        witness,
        corpus: Corpus::from_witness(witness),
        token_surface: token.surface.clone(),
        token_normalized: token.normalized.clone(),
        lemma: lemma.to_string(),
        morphology: token.morph.clone(),
        position,
        verse_text: verse_text.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pos_derived_from_tags() {
        assert_eq!(
            Morphology("N2--ASM---".into()).part_of_speech(),
            Some(PartOfSpeech::Noun)
        );
        assert_eq!(
            Morphology("V1--PMS2S-".into()).part_of_speech(),
            Some(PartOfSpeech::Verb)
        );
        assert_eq!(
            Morphology("RA--ASF---".into()).part_of_speech(),
            Some(PartOfSpeech::Article)
        );
        assert_eq!(
            Morphology("RP--GS----".into()).part_of_speech(),
            Some(PartOfSpeech::Pronoun)
        );
        assert_eq!(
            Morphology("C---------".into()).part_of_speech(),
            Some(PartOfSpeech::Conjunction)
        );
        assert_eq!(
            Morphology("P---------".into()).part_of_speech(),
            Some(PartOfSpeech::Preposition)
        );
        assert_eq!(
            Morphology("D---------".into()).part_of_speech(),
            Some(PartOfSpeech::Adverb)
        );
        assert_eq!(
            Morphology("A1--APM---".into()).part_of_speech(),
            Some(PartOfSpeech::Adjective)
        );
    }

    #[test]
    fn descriptions_decode_packard_tags() {
        assert_eq!(
            Morphology("N2--ASM---".into()).description(),
            Some("noun acc sg masc".to_string())
        );
        assert_eq!(
            Morphology("N2N-VSN---".into()).description(),
            Some("noun voc sg neut".to_string())
        );
        assert_eq!(
            Morphology("V1--PMS2S-".into()).description(),
            Some("verb pres mid subj 2 sg".to_string())
        );
        assert_eq!(
            Morphology("VA--AAD2S-".into()).description(),
            Some("verb aor act imper 2 sg".to_string())
        );
        assert_eq!(
            Morphology("V1--PAPAPM".into()).description(),
            Some("verb pres act ptcp acc pl masc".to_string())
        );
        assert_eq!(
            Morphology("V2-PAPNSN-".into()).description(),
            Some("verb pres act ptcp nom sg neut".to_string())
        );
        assert_eq!(
            Morphology("RA--ASF---".into()).description(),
            Some("article acc sg fem".to_string())
        );
        assert_eq!(
            Morphology("RP--GS----".into()).description(),
            Some("pronoun gen sg".to_string())
        );
        assert_eq!(
            Morphology("C---------".into()).description(),
            Some("conjunction".to_string())
        );
    }

    #[test]
    fn unknown_tags_are_tolerated() {
        assert_eq!(Morphology("".into()).part_of_speech(), None);
        assert_eq!(Morphology("!!!".into()).part_of_speech(), None);
        assert_eq!(Morphology("".into()).description(), None);
    }

    #[test]
    fn corpus_labels_distinguish_future_corpora() {
        assert_eq!(Corpus::Apocrypha.label(), "apocrypha");
        assert_eq!(Corpus::SeptuagintOt.label(), "septuagint_ot");
        assert_eq!(Corpus::NewTestament.label(), "new_testament");
        assert_eq!(Corpus::from_witness(WitnessId::Lxx), Corpus::Apocrypha);
    }
}
