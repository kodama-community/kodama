// Ported from moonbit inspect/unicode_symbol_test.mbt
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
                &parse("\\alpha_2^n + \\beta", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "α₂ⁿ + β"
        );
    }

    #[test]
    fn t1() {
        assert_eq!(
            render(
                &parse(
                    "\\chi(X, E) = \\int_X \\operatorname{ch}(E) \\cdot \\operatorname{td}(T_X)",
                    &mut Settings::new()
                )
                .unwrap(),
                RenderConfig::new()
            ),
            "χ(X, E) = ∫_X ch(E) ⋅ td(T_X)"
        );
    }

    #[test]
    fn t2() {
        assert_eq!(
            render(
                &parse(
                    "\\frac{a}{b}+\\sqrt[3]{x}+\\overline y",
                    &mut Settings::new()
                )
                .unwrap(),
                RenderConfig::new()
            ),
            "a∕b + ³√x + overline(y)"
        );
    }

    #[test]
    fn t3() {
        assert_eq!(
            render(
                &parse("\\not=", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "≠"
        );
        assert_eq!(
            render(
                &parse("\\not<", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "≮"
        );
        assert_eq!(
            render(
                &parse("\\not\\in", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "∉"
        );
        assert_eq!(
            render(
                &parse("\\not x", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "x̸"
        );
        assert_eq!(
            render(
                &parse("a\\not=b", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "a ≠ b"
        );
        assert_eq!(
            render(
                &parse("\\neq", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "≠"
        );
        assert_eq!(
            render(
                &parse("\\notin", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "∉"
        );
    }

    #[test]
    fn t4() {
        assert_eq!(
            render(
                &parse("x^{q}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "x^q"
        );
    }

    #[test]
    fn t5() {
        assert_eq!(
            render(
                &parse("\\sqrt[q]{x}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "root(q,x)"
        );
    }

    #[test]
    fn t6() {
        assert_eq!(
            render(
                &parse("\\sqrt{x}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "√x"
        );
        assert_eq!(
            render(
                &parse("\\sqrt{x+y}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "√(x + y)"
        );
    }

    #[test]
    fn t7() {
        assert_eq!(
            render(
                &parse("\\sqrt{\\frac{a}{b}}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "√(a∕b)"
        );
        assert_eq!(
            render(
                &parse("\\sqrt[3]{\\frac{a}{b}}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "³√(a∕b)"
        );
        assert_eq!(
            render(
                &parse("\\sqrt{\\frac{\\frac{a}{b}}{c}}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "√((a∕b)∕c)"
        );
    }

    #[test]
    fn t8() {
        let mut settings = Settings {
            display_mode: true,
            ..Settings::new()
        };
        assert_eq!(
            render(
                &parse("\\sqrt{\\frac{\\pi e}{2}}", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            "   πe  \n√(────)\n   2   "
        );
        assert_eq!(
            render(
                &parse("x + \\sqrt{\\frac{a}{b}}", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            "       a  \nx + √(───)\n       b  "
        );
    }
}
