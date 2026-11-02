// Ported from moonbit inspect/unicode_spacing_test.mbt
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
        let mut settings = Settings {
            display_mode: true,
            ..Settings::new()
        };
        assert_eq!(
            render(
                &parse(
                    "\\sum_{n\\ge1}\\frac{1}{n^2}=\\frac{\\pi^2}{6}",
                    &mut settings
                )
                .unwrap(),
                RenderConfig::new()
            ),
            "     1      π² \n ∑  ──── = ────\nn≥1  n²     6  "
        );
    }

    #[test]
    fn t1() {
        assert_eq!(
            render(
                &parse("a\\,b", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "a b"
        );
        assert_eq!(
            render(
                &parse("a\\:b", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "a b"
        );
        assert_eq!(
            render(
                &parse("a\\;b", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "a b"
        );
        assert_eq!(
            render(
                &parse("a\\enspace b", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "a b"
        );
        assert_eq!(
            render(
                &parse("a\\quad b", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "a  b"
        );
        assert_eq!(
            render(
                &parse("a\\qquad b", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "a    b"
        );
        assert_eq!(
            render(
                &parse("a\\!b", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "ab"
        );
        assert_eq!(
            render(
                &parse("\\int\\!dx", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "∫ dx"
        );
        assert_eq!(
            render(
                &parse("x\\kern{1pt}y", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "xkern(1pt)y"
        );
    }

    #[test]
    fn t2() {
        assert_eq!(
            render(
                &parse(
                    "\\R_{\\ge 0}^* = \\R_{\\ge 0} \\cup \\{\\infty\\}",
                    &mut Settings::new()
                )
                .unwrap(),
                RenderConfig::new()
            ),
            "ℝ_(≥0)^∗ = ℝ_(≥0) ∪ {∞}"
        );
    }

    #[test]
    fn t3() {
        assert_eq!(
            render(
                &parse("A'", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "A'"
        );
        assert_eq!(
            render(
                &parse("A^\\prime", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "A'"
        );
        assert_eq!(
            render(
                &parse("A''", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "A''"
        );
        assert_eq!(
            render(
                &parse("A^{\\prime\\prime}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "A''"
        );
        assert_eq!(
            render(
                &parse("f'(x)", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "f'(x)"
        );
        assert_eq!(
            render(
                &parse("A'^2", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "A'²"
        );
        assert_eq!(
            render(
                &parse("A\\prime", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "A′"
        );
    }

    #[test]
    fn t4() {
        assert_eq!(
            render(
                &parse("a+b", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "a + b"
        );
        assert_eq!(
            render(
                &parse("a=b", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "a = b"
        );
        assert_eq!(
            render(
                &parse("x_{a+b}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "x_(a+b)"
        );
        assert_eq!(
            render(
                &parse("x^{a+b}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "xᵃ⁺ᵇ"
        );
        assert_eq!(
            render(
                &parse("{+x}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "+x"
        );
        assert_eq!(
            render(
                &parse("a + b = c \\cup d", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "a + b = c ∪ d"
        );
    }
}
