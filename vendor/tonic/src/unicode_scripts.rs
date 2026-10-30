use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Whether a Unicode script is a subscript or superscript.
pub enum UnicodeScriptKind {
    UnicodeSubscript,
    UnicodeSuperscript,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnicodeScript {
    pub replacement: String,
    pub kind: UnicodeScriptKind,
}

fn unicode_scripts() -> &'static HashMap<String, UnicodeScript> {
    static TABLE: OnceLock<HashMap<String, UnicodeScript>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut map = HashMap::new();
        fn sub(map: &mut HashMap<String, UnicodeScript>, c: &str, r: &str) {
            map.insert(
                c.to_string(),
                UnicodeScript {
                    replacement: r.to_string(),
                    kind: UnicodeScriptKind::UnicodeSubscript,
                },
            );
        }
        fn sup(map: &mut HashMap<String, UnicodeScript>, c: &str, r: &str) {
            map.insert(
                c.to_string(),
                UnicodeScript {
                    replacement: r.to_string(),
                    kind: UnicodeScriptKind::UnicodeSuperscript,
                },
            );
        }
        sub(&mut map, "₊", "+");
        sub(&mut map, "₋", "-");
        sub(&mut map, "₌", "=");
        sub(&mut map, "₍", "(");
        sub(&mut map, "₎", ")");
        sub(&mut map, "₀", "0");
        sub(&mut map, "₁", "1");
        sub(&mut map, "₂", "2");
        sub(&mut map, "₃", "3");
        sub(&mut map, "₄", "4");
        sub(&mut map, "₅", "5");
        sub(&mut map, "₆", "6");
        sub(&mut map, "₇", "7");
        sub(&mut map, "₈", "8");
        sub(&mut map, "₉", "9");
        sub(&mut map, "ₐ", "a");
        sub(&mut map, "ₑ", "e");
        sub(&mut map, "ₕ", "h");
        sub(&mut map, "ᵢ", "i");
        sub(&mut map, "ⱼ", "j");
        sub(&mut map, "ₖ", "k");
        sub(&mut map, "ₗ", "l");
        sub(&mut map, "ₘ", "m");
        sub(&mut map, "ₙ", "n");
        sub(&mut map, "ₒ", "o");
        sub(&mut map, "ₚ", "p");
        sub(&mut map, "ᵣ", "r");
        sub(&mut map, "ₛ", "s");
        sub(&mut map, "ₜ", "t");
        sub(&mut map, "ᵤ", "u");
        sub(&mut map, "ᵥ", "v");
        sub(&mut map, "ₓ", "x");
        sub(&mut map, "ᵦ", "β");
        sub(&mut map, "ᵧ", "γ");
        sub(&mut map, "ᵨ", "ρ");
        sub(&mut map, "ᵩ", "ϕ");
        sub(&mut map, "ᵪ", "χ");
        sup(&mut map, "⁺", "+");
        sup(&mut map, "⁻", "-");
        sup(&mut map, "⁼", "=");
        sup(&mut map, "⁽", "(");
        sup(&mut map, "⁾", ")");
        sup(&mut map, "⁰", "0");
        sup(&mut map, "¹", "1");
        sup(&mut map, "²", "2");
        sup(&mut map, "³", "3");
        sup(&mut map, "⁴", "4");
        sup(&mut map, "⁵", "5");
        sup(&mut map, "⁶", "6");
        sup(&mut map, "⁷", "7");
        sup(&mut map, "⁸", "8");
        sup(&mut map, "⁹", "9");
        sup(&mut map, "ᴬ", "A");
        sup(&mut map, "ᴮ", "B");
        sup(&mut map, "ᴰ", "D");
        sup(&mut map, "ᴱ", "E");
        sup(&mut map, "ᴳ", "G");
        sup(&mut map, "ᴴ", "H");
        sup(&mut map, "ᴵ", "I");
        sup(&mut map, "ᴶ", "J");
        sup(&mut map, "ᴷ", "K");
        sup(&mut map, "ᴸ", "L");
        sup(&mut map, "ᴹ", "M");
        sup(&mut map, "ᴺ", "N");
        sup(&mut map, "ᴼ", "O");
        sup(&mut map, "ᴾ", "P");
        sup(&mut map, "ᴿ", "R");
        sup(&mut map, "ᵀ", "T");
        sup(&mut map, "ᵁ", "U");
        sup(&mut map, "ⱽ", "V");
        sup(&mut map, "ᵂ", "W");
        sup(&mut map, "ᵃ", "a");
        sup(&mut map, "ᵇ", "b");
        sup(&mut map, "ᶜ", "c");
        sup(&mut map, "ᵈ", "d");
        sup(&mut map, "ᵉ", "e");
        sup(&mut map, "ᶠ", "f");
        sup(&mut map, "ᵍ", "g");
        sup(&mut map, "ʰ", "h");
        sup(&mut map, "ⁱ", "i");
        sup(&mut map, "ʲ", "j");
        sup(&mut map, "ᵏ", "k");
        sup(&mut map, "ˡ", "l");
        sup(&mut map, "ᵐ", "m");
        sup(&mut map, "ⁿ", "n");
        sup(&mut map, "ᵒ", "o");
        sup(&mut map, "ᵖ", "p");
        sup(&mut map, "ʳ", "r");
        sup(&mut map, "ˢ", "s");
        sup(&mut map, "ᵗ", "t");
        sup(&mut map, "ᵘ", "u");
        sup(&mut map, "ᵛ", "v");
        sup(&mut map, "ʷ", "w");
        sup(&mut map, "ˣ", "x");
        sup(&mut map, "ʸ", "y");
        sup(&mut map, "ᶻ", "z");
        sup(&mut map, "ᵝ", "β");
        sup(&mut map, "ᵞ", "γ");
        sup(&mut map, "ᵟ", "δ");
        sup(&mut map, "ᵠ", "ϕ");
        sup(&mut map, "ᵡ", "χ");
        sup(&mut map, "ᶿ", "θ");
        map
    })
}

pub(crate) fn lookup_unicode_script(text: &str) -> Option<&'static UnicodeScript> {
    unicode_scripts().get(text)
}

/// Returns the Unicode superscript or subscript character for `replacement`.
pub fn unicode_script_character(kind: UnicodeScriptKind, replacement: &str) -> Option<String> {
    unicode_scripts()
        .iter()
        .find(|(_, script)| script.kind == kind && script.replacement == replacement)
        .map(|(character, _)| character.clone())
}

/// True when the codepoint falls within one of the supported Unicode script
/// blocks. Mirrors KaTeX's `supportedCodepoint` (unicodeScripts.ts).
pub fn supported_codepoint(code: u32) -> bool {
    (0x0100..=0x024F).contains(&code) || // Latin Extended-A/B
    (0x0300..=0x036F).contains(&code) || // Combining Diacritical marks
    (0x0400..=0x04FF).contains(&code) || // Cyrillic
    (0x0530..=0x058F).contains(&code) || // Armenian
    (0x0900..=0x109F).contains(&code) || // Brahmic
    (0x10A0..=0x10FF).contains(&code) || // Georgian
    (0x3000..=0x30FF).contains(&code) || // CJK symbols, Hiragana, Katakana
    (0x4E00..=0x9FAF).contains(&code) || // CJK ideograms
    (0xFF00..=0xFF60).contains(&code) || // Fullwidth punctuation
    (0xAC00..=0xD7AF).contains(&code) // Hangul
}
