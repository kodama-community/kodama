/// Returns the Unicode mathematical alphanumeric character for a given font
/// style and ASCII letter/digit, or `None` when no mapping exists.
///
/// Font styles that match LaTeX commands:
/// - `mathbf` / `boldsymbol` — Bold / Bold Italic
/// - `mathit` — Italic
/// - `mathbb` — Double-struck (blackboard bold)
/// - `mathcal` — Calligraphic / Script
/// - `mathfrak` — Fraktur
/// - `mathscr` — Script (rsfs)
/// - `mathsf` — Sans-serif
/// - `mathtt` — Monospace
/// - `mathsfit` — Sans-serif Italic
/// - `mathrm` / `mathnormal` — Roman (identity, not handled here) / Italic
pub fn unicode_font_character(font: &str, ch: &str) -> Option<String> {
    let mut chars = ch.chars();
    let c = chars.next()?;
    if chars.next().is_some() {
        return None;
    }
    let cp = match font {
        "mathbf" | "boldsymbol" => {
            let base = if font == "boldsymbol" {
                0x1D468
            } else {
                0x1D400
            };
            let greek_cap = if font == "boldsymbol" {
                0x1D71C
            } else {
                0x1D6A8
            };
            let greek_small = if font == "boldsymbol" {
                0x1D736
            } else {
                0x1D6C2
            };
            let greek_symbols = if font == "boldsymbol" {
                0x1D750
            } else {
                0x1D6DC
            };
            let cd = c as u32;
            match c {
                'A'..='Z' => Some(base + cd - 'A' as u32),
                'a'..='z' => Some(base + 26 + cd - 'a' as u32),
                '0'..='9' if font != "boldsymbol" => Some(0x1D7CE + cd - '0' as u32),
                'Α'..='Ω' => Some(greek_cap + cd - 'Α' as u32),
                'α'..='ω' => Some(greek_small + cd - 'α' as u32),
                'ϵ' => Some(greek_symbols),
                'ϑ' => Some(greek_symbols + 1),
                'ϰ' => Some(greek_symbols + 2),
                'ϕ' => Some(greek_symbols + 3),
                'ϱ' => Some(greek_symbols + 4),
                'ϖ' => Some(greek_symbols + 5),
                '∇' => Some(greek_cap + 0x19),
                '∂' => Some(greek_small + 0x19),
                _ => None,
            }
        }
        "mathit" | "mathnormal" => {
            let cd = c as u32;
            match c {
                'A'..='Z' => Some(0x1D434 + cd - 'A' as u32),
                'a'..='z' => Some(0x1D44E + cd - 'a' as u32),
                'Α'..='Ω' => Some(0x1D6E2 + cd - 'Α' as u32),
                'α'..='ω' => Some(0x1D6FC + cd - 'α' as u32),
                'ϵ' => Some(0x1D716),
                'ϑ' => Some(0x1D717),
                'ϰ' => Some(0x1D718),
                'ϕ' => Some(0x1D719),
                'ϱ' => Some(0x1D71A),
                'ϖ' => Some(0x1D71B),
                '∇' => Some(0x1D6FB),
                '∂' => Some(0x1D715),
                _ => None,
            }
        }
        "mathsf" => {
            let cd = c as u32;
            match c {
                'A'..='Z' => Some(0x1D5A0 + cd - 'A' as u32),
                'a'..='z' => Some(0x1D5BA + cd - 'a' as u32),
                '0'..='9' => Some(0x1D7E2 + cd - '0' as u32),
                _ => None,
            }
        }
        "mathtt" => {
            let cd = c as u32;
            match c {
                'A'..='Z' => Some(0x1D670 + cd - 'A' as u32),
                'a'..='z' => Some(0x1D68A + cd - 'a' as u32),
                '0'..='9' => Some(0x1D7F6 + cd - '0' as u32),
                _ => None,
            }
        }
        "mathbb" => {
            let cd = c as u32;
            match c {
                'A' => Some(0x1D538),
                'B' => Some(0x1D539),
                'C' => Some(0x2102),
                'D' => Some(0x1D53B),
                'E' => Some(0x1D53C),
                'F' => Some(0x1D53D),
                'G' => Some(0x1D53E),
                'H' => Some(0x210D),
                'I' => Some(0x1D540),
                'J' => Some(0x1D541),
                'K' => Some(0x1D542),
                'L' => Some(0x1D543),
                'M' => Some(0x1D544),
                'N' => Some(0x2115),
                'O' => Some(0x1D546),
                'P' => Some(0x2119),
                'Q' => Some(0x211A),
                'R' => Some(0x211D),
                'S' => Some(0x1D54A),
                'T' => Some(0x1D54B),
                'U' => Some(0x1D54C),
                'V' => Some(0x1D54D),
                'W' => Some(0x1D54E),
                'X' => Some(0x1D54F),
                'Y' => Some(0x1D550),
                'Z' => Some(0x2124),
                'a'..='z' => Some(0x1D552 + cd - 'a' as u32),
                '0'..='9' => Some(0x1D7D8 + cd - '0' as u32),
                _ => None,
            }
        }
        "mathcal" => match c {
            'A' => Some(0x1D49C),
            'B' => Some(0x212C),
            'C' => Some(0x1D49E),
            'D' => Some(0x1D49F),
            'E' => Some(0x2130),
            'F' => Some(0x2131),
            'G' => Some(0x1D4A2),
            'H' => Some(0x210B),
            'I' => Some(0x2110),
            'J' => Some(0x1D4A5),
            'K' => Some(0x1D4A6),
            'L' => Some(0x2112),
            'M' => Some(0x2133),
            'N' => Some(0x1D4A9),
            'O' => Some(0x1D4AA),
            'P' => Some(0x1D4AB),
            'Q' => Some(0x1D4AC),
            'R' => Some(0x211B),
            'S' => Some(0x1D4AE),
            'T' => Some(0x1D4AF),
            'U' => Some(0x1D4B0),
            'V' => Some(0x1D4B1),
            'W' => Some(0x1D4B2),
            'X' => Some(0x1D4B3),
            'Y' => Some(0x1D4B4),
            'Z' => Some(0x1D4B5),
            _ => None,
        },
        "mathfrak" => {
            let cd = c as u32;
            match c {
                'A' => Some(0x1D504),
                'B' => Some(0x1D505),
                'C' => Some(0x212D),
                'D' => Some(0x1D507),
                'E' => Some(0x1D508),
                'F' => Some(0x1D509),
                'G' => Some(0x1D50A),
                'H' => Some(0x210C),
                'I' => Some(0x2111),
                'J' => Some(0x1D50D),
                'K' => Some(0x1D50E),
                'L' => Some(0x1D50F),
                'M' => Some(0x1D510),
                'N' => Some(0x1D511),
                'O' => Some(0x1D512),
                'P' => Some(0x1D513),
                'Q' => Some(0x1D514),
                'R' => Some(0x211C),
                'S' => Some(0x1D516),
                'T' => Some(0x1D517),
                'U' => Some(0x1D518),
                'V' => Some(0x1D519),
                'W' => Some(0x1D51A),
                'X' => Some(0x1D51B),
                'Y' => Some(0x1D51C),
                'Z' => Some(0x2128),
                'a'..='z' => Some(0x1D51E + cd - 'a' as u32),
                _ => None,
            }
        }
        "mathscr" => {
            let cd = c as u32;
            match c {
                'A'..='Z' => Some(0x1D4D0 + cd - 'A' as u32),
                'a'..='z' => Some(0x1D4EA + cd - 'a' as u32),
                _ => None,
            }
        }
        "mathsfit" => {
            let cd = c as u32;
            match c {
                'A'..='Z' => Some(0x1D608 + cd - 'A' as u32),
                'a'..='z' => Some(0x1D622 + cd - 'a' as u32),
                _ => None,
            }
        }
        _ => None,
    };
    cp.map(|cp| char::from_u32(cp).unwrap().to_string())
}
