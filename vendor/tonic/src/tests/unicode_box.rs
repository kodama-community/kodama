// Ported from moonbit inspect/unicode_box_test.mbt
#![allow(unused_imports)]
use crate::parse;
use crate::settings::{Macros, Settings};
use crate::unicode::render;
use crate::unicode::{LineStyle, RenderConfig};
use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t0() {
        assert_eq!(
            render(
                &parse("\\boxed{x}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "┌─┐\n│x│\n└─┘"
        );
        assert_eq!(
            render(
                &parse("\\boxed{\\pi=\\frac c d}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "┌───────┐\n│     c │\n│π = ───│\n│     d │\n└───────┘"
        );
    }

    #[test]
    fn t1() {
        let mut settings = Settings {
            display_mode: true,
            ..Settings::new()
        };
        assert_eq!(
            render(
                &parse("x = \\boxed{\\pi=\\frac c d}", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            "    ┌───────┐\n    │     c │\nx = │π = ───│\n    │     d │\n    └───────┘"
        );
    }

    #[test]
    fn t2() {
        let config = RenderConfig {
            line_style: LineStyle::Ascii,
            ..RenderConfig::new()
        };
        let mut settings = Settings {
            display_mode: true,
            ..Settings::new()
        };
        assert_eq!(
            render(
                &parse("\\boxed{x}", &mut Settings::new()).unwrap(),
                config.clone()
            ),
            "---\n|x|\n---"
        );
        assert_eq!(
            render(
                &parse("\\boxed{\\pi=\\frac c d}", &mut Settings::new()).unwrap(),
                config.clone()
            ),
            "---------\n|     c |\n|π = ---|\n|     d |\n---------"
        );
        assert_eq!(
            render(
                &parse("\\frac{a}{b}", &mut settings).unwrap(),
                config.clone()
            ),
            " a \n---\n b "
        );
    }
}
