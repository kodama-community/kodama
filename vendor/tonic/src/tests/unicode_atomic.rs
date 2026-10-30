// Ported from moonbit inspect/unicode_atomic_test.mbt
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
                &parse("\\frac{a+b}{c}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "(a+b)∕c"
        );
        assert_eq!(
            render(
                &parse("{\\frac{a}{b}}^2", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "(a∕b)²"
        );
    }

    #[test]
    fn t1() {
        assert_eq!(
            render(
                &parse("\\sqrt{x}_1", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "√x₁"
        );
        assert_eq!(
            render(
                &parse("\\sqrt{x}^2", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "√x²"
        );
        assert_eq!(
            render(
                &parse("\\sqrt{x+y}_1", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "√(x + y)₁"
        );
        assert_eq!(
            render(
                &parse("\\frac{\\sqrt{x}}{y}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "√x∕y"
        );
    }

    #[test]
    fn t2() {
        assert_eq!(
            render(
                &parse("\\overline{x}_1", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "overline(x)₁"
        );
        assert_eq!(
            render(
                &parse("\\underline{x}^2", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "underline(x)²"
        );
        assert_eq!(
            render(
                &parse("\\frac{\\overline{a}}{b}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "overline(a)∕b"
        );
    }

    #[test]
    fn t3() {
        assert_eq!(
            render(
                &parse("\\phantom{x}_1", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "phantom(x)₁"
        );
        assert_eq!(
            render(
                &parse("\\cancel{x}_1", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "cancel(x)₁"
        );
        assert_eq!(
            render(
                &parse("\\pmb{x}_1", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "bold(x)₁"
        );
        assert_eq!(
            render(
                &parse("\\rule{1em}{2pt}_1", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "rule(1em,2pt)₁"
        );
        assert_eq!(
            render(
                &parse("\\kern{1pt}_1", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "kern(1pt)₁"
        );
        assert_eq!(
            render(
                &parse("\\underleftarrow{x}_1", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "underaccent(underleftarrow,x)₁"
        );
    }

    #[test]
    fn t4() {
        assert_eq!(
            render(
                &parse("\\frac{a}{b}_1", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "(a∕b)₁"
        );
    }

    #[test]
    fn t5() {
        assert_eq!(
            render(
                &parse("\\frac{a}{b}_1", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "(a∕b)₁"
        );
        assert_eq!(
            render(
                &parse("{a+b}_1", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "(a + b)₁"
        );
        assert_eq!(
            render(
                &parse("\\frac{a+b}{c+d}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "(a+b)∕(c+d)"
        );
    }

    #[test]
    fn t6() {
        assert_eq!(
            render(
                &parse("\\hat{x}_1", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "x̂₁"
        );
        assert_eq!(
            render(
                &parse("\\hat{\\R}_1", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "ℝ̂₁"
        );
        assert_eq!(
            render(
                &parse("\\bar{x}_1", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "x̄₁"
        );
    }

    #[test]
    fn t7() {
        assert_eq!(
            render(
                &parse("\\text{x}_1", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "x₁"
        );
        assert_eq!(
            render(
                &parse("\\mathrm{x}_1", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "x₁"
        );
        assert_eq!(
            render(
                &parse("\\textbf{x}_1", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "x₁"
        );
    }

    #[test]
    fn t8() {
        assert_eq!(
            render(
                &parse("\\text{ab}_1", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "(ab)₁"
        );
        assert_eq!(
            render(
                &parse("\\text{a+b}_1", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "(a+b)₁"
        );
    }

    #[test]
    fn t9() {
        let mut settings = Settings {
            display_mode: true,
            ..Settings::new()
        };
        assert_eq!(
            render(
                &parse("x^{\\frac{a}{b}}", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            "x^(a∕b)"
        );
        assert_eq!(
            render(
                &parse("x_{\\frac{a}{b}}", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            "x_(a∕b)"
        );
    }

    #[test]
    fn t10() {
        assert_eq!(
            render(
                &parse("a\\mathchoice{D}{T}{S}{SS}b", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "aTb"
        );
        assert_eq!(
            render(
                &parse(
                    "a\\textstyle\\mathchoice{D}{T}{S}{SS}b",
                    &mut Settings::new()
                )
                .unwrap(),
                RenderConfig::new()
            ),
            "aTb"
        );
        assert_eq!(
            render(
                &parse(
                    "a\\displaystyle\\mathchoice{D}{T}{S}{SS}b",
                    &mut Settings::new()
                )
                .unwrap(),
                RenderConfig::new()
            ),
            "aDb"
        );
        assert_eq!(
            render(
                &parse(
                    "a\\scriptstyle\\mathchoice{D}{T}{S}{SS}b",
                    &mut Settings::new()
                )
                .unwrap(),
                RenderConfig::new()
            ),
            "aSb"
        );
        assert_eq!(
            render(
                &parse(
                    "a\\scriptscriptstyle\\mathchoice{D}{T}{S}{SS}b",
                    &mut Settings::new()
                )
                .unwrap(),
                RenderConfig::new()
            ),
            "aSSb"
        );
    }

    #[test]
    fn t11() {
        let mut settings = Settings {
            display_mode: true,
            ..Settings::new()
        };
        assert_eq!(
            render(
                &parse("\\overline{\\frac{a}{b}}", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            "          a  \noverline(───)\n          b  "
        );
    }
}
