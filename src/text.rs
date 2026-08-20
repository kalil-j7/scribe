//! Text normalization and tokenization shared by the store, search, and
//! importers.
//!
//! Rules:
//! * NFC canonical composition.
//! * Unicode lowercase.
//! * Greek accents/breathings removed (NFD + drop combining marks), so
//!   `πειρασμός` and `πειρασμος` match.
//! * Greek final sigma `ς` folds to `σ`.
//! * Non-alphanumeric characters are dropped (so `Lord,` matches `lord`).
//! * Internal apostrophes are kept (`king's` stays `king's`).

use unicode_normalization::UnicodeNormalization;

/// Normalize one token/word for matching and indexing.
pub fn normalize(s: &str) -> String {
    let nfc: String = s.nfkc().collect();
    let lower = nfc.to_lowercase();
    let mut out = String::with_capacity(lower.len());
    for ch in lower.nfd() {
        if is_combining_mark(ch) {
            continue;
        }
        let ch = match ch {
            'ς' => 'σ',
            '’' => '\'',
            _ => ch,
        };
        if ch.is_alphanumeric() || ch == '\'' {
            out.push(ch);
        }
    }
    out
}

fn is_combining_mark(c: char) -> bool {
    matches!(c as u32, 0x0300..=0x036F | 0x1AB0..=0x1AFF | 0x1DC0..=0x1DFF | 0x20D0..=0x20FF | 0xFE20..=0xFE2F)
}

/// Split a verse text into normalized tokens (deduplicated, in order).
pub fn tokenize(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for word in text.split_whitespace() {
        let n = normalize(word);
        if !n.is_empty() && !out.contains(&n) {
            out.push(n);
        }
    }
    out
}

/// Normalize a multi-word search query into terms (deduplicated).
pub fn query_terms(query: &str) -> Vec<String> {
    tokenize(query)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folds_case_and_punctuation() {
        assert_eq!(normalize("Lord,"), "lord");
        assert_eq!(normalize("Lord"), "lord");
        assert_eq!(normalize("temptation."), "temptation");
        assert_eq!(normalize("king’s"), "king's"); // curly apostrophe -> ascii
    }

    #[test]
    fn strips_greek_accents_and_folds_final_sigma() {
        // Note: expected strings use regular sigma U+03C3 (final sigma folds).
        assert_eq!(normalize("πειρασμός"), "πειρασμο\u{3c3}");
        assert_eq!(normalize("φόβος"), "φοβο\u{3c3}");
        assert_eq!(normalize("κυρίῳ,"), "κυριω");
        assert_eq!(normalize("προσέρχῃ"), "προσερχη");
    }

    #[test]
    fn query_terms_split() {
        assert_eq!(
            query_terms("fear of the Lord"),
            vec!["fear", "of", "the", "lord"]
        );
        assert!(query_terms("!!!").is_empty());
    }
}
