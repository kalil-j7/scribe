//! Simple scholarly-style Greek → Latin transliteration.
//!
//! Accents and breathings are ignored (the input is normalized first).
//! Scheme: plain letter mapping with the standard digraphs
//! θ→th, φ→ph, χ→ch, ψ→ps; η→e, ω→o, υ→u (diphthongs fall out naturally,
//! e.g. αυ→au, ει→ei). Examples: `πειρασμός` → `peirasmos`,
//! `φόβος` → `phobos`, `ἀγαπάω` → `agapao`.

use unicode_normalization::UnicodeNormalization;

/// Transliterate a Greek word to Latin letters.
pub fn transliterate(s: &str) -> String {
    let nfc: String = s.nfkc().collect();
    let lower = nfc.to_lowercase();
    let mut out = String::with_capacity(lower.len());
    for ch in lower.nfd() {
        if is_combining_mark(ch) {
            continue;
        }
        let l = match ch {
            'α' => "a",
            'β' => "b",
            'γ' => "g",
            'δ' => "d",
            'ε' => "e",
            'ζ' => "z",
            'η' => "e",
            'θ' => "th",
            'ι' => "i",
            'κ' => "k",
            'λ' => "l",
            'μ' => "m",
            'ν' => "n",
            'ξ' => "x",
            'ο' => "o",
            'π' => "p",
            'ρ' => "r",
            'σ' | 'ς' => "s",
            'τ' => "t",
            'υ' => "u",
            'φ' => "ph",
            'χ' => "ch",
            'ψ' => "ps",
            'ω' => "o",
            'ϝ' => "w",
            _ => continue,
        };
        out.push_str(l);
    }
    out
}

fn is_combining_mark(c: char) -> bool {
    matches!(c as u32, 0x0300..=0x036F | 0x1AB0..=0x1AFF | 0x1DC0..=0x1DFF | 0x20D0..=0x20FF | 0xFE20..=0xFE2F)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transliterates_peirasmos() {
        assert_eq!(transliterate("πειρασμός"), "peirasmos");
    }

    #[test]
    fn transliterates_phobos() {
        assert_eq!(transliterate("φόβος"), "phobos");
    }

    #[test]
    fn transliterates_agapao() {
        assert_eq!(transliterate("ἀγαπάω"), "agapao");
    }

    #[test]
    fn transliterates_uppercase() {
        assert_eq!(transliterate("ΠΕΙΡΑΣΜΟΣ"), "peirasmos");
    }

    #[test]
    fn transliterates_diphthongs() {
        assert_eq!(transliterate("αὐτός"), "autos");
        assert_eq!(transliterate("εἰμί"), "eimi");
        assert_eq!(transliterate("χριστός"), "christos");
    }
}
