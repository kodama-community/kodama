// Ported from moonbit inspect/unicode_basic_test.mbt
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
                &parse("\\overline{xy}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "overline(xy)"
        );
        assert_eq!(
            render(
                &parse("\\underline{x}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "underline(x)"
        );
    }

    #[test]
    fn t1() {
        assert_eq!(
            render(
                &parse("\\left( x \\right)", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "(x)"
        );
        assert_eq!(
            render(
                &parse("\\left[ x \\right]", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "[x]"
        );
    }

    #[test]
    fn t2() {
        assert_eq!(
            render(
                &parse("\\mathbb{R}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "ℝ"
        );
        assert_eq!(
            render(
                &parse("\\mathbf{x}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "𝐱"
        );
        assert_eq!(
            render(
                &parse("\\mathit{x}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "𝑥"
        );
        assert_eq!(
            render(
                &parse("\\mathsf{A}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "𝖠"
        );
        assert_eq!(
            render(
                &parse("\\mathtt{x}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "𝚡"
        );
        assert_eq!(
            render(
                &parse("\\mathcal{L}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "ℒ"
        );
    }

    #[test]
    fn t3() {
        assert_eq!(
            render(
                &parse("\\mathbb{1}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "𝟙"
        );
        assert_eq!(
            render(
                &parse("\\mathbb{42}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "𝟜𝟚"
        );
        assert_eq!(
            render(
                &parse("\\mathit{\\alpha}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "𝛼"
        );
        assert_eq!(
            render(
                &parse("\\mathit{\\Gamma\\beta}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "𝛤𝛽"
        );
        assert_eq!(
            render(
                &parse("\\mathbf{\\nabla\\partial}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "𝛁𝛛"
        );
        assert_eq!(
            render(
                &parse("\\mathit{\\nabla\\partial}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "𝛻𝜕"
        );
        assert_eq!(
            render(
                &parse("\\mathnormal{x}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "𝑥"
        );
        assert_eq!(
            render(
                &parse("\\mathnormal{\\alpha}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "𝛼"
        );
        assert_eq!(
            render(
                &parse("\\mathrm{x}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "x"
        );
    }

    #[test]
    fn t4() {
        assert_eq!(
            render(
                &parse("\\text{hello}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "hello"
        );
        assert_eq!(
            render(
                &parse("\\phantom{xy}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "phantom(xy)"
        );
    }

    #[test]
    fn t5() {
        assert_eq!(
            render(
                &parse("\\phantom{x}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "phantom(x)"
        );
        assert_eq!(
            render(
                &parse("\\vphantom{x}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "vphantom(x)"
        );
        assert_eq!(
            render(
                &parse("\\hphantom{x}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "phantom(x)"
        );
        assert_eq!(
            render(
                &parse("\\vphantom{a+b}_1", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "vphantom(a + b)₁"
        );
    }

    #[test]
    fn t6() {
        assert_eq!(
            render(
                &parse("\\smash{x}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "x"
        );
        assert_eq!(
            render(
                &parse("\\raisebox{1pt}{x}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "x"
        );
        assert_eq!(
            render(
                &parse("\\mathllap{x}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "x"
        );
        assert_eq!(
            render(
                &parse("\\vcenter{x}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "x"
        );
    }

    #[test]
    fn t7() {
        assert_eq!(
            render(
                &parse("\\href{a}{b}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "b"
        );
        assert_eq!(
            render(
                &parse("\\url{http://x}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "http://x"
        );
        assert_eq!(
            render(
                &parse("\\includegraphics{img.png}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "img"
        );
        assert_eq!(
            render(
                &parse("\\htmlClass{foo}{x}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "x"
        );
        assert_eq!(
            render(
                &parse(
                    "\\nosuchcommand",
                    &mut Settings {
                        throw_on_error: false,
                        ..Settings::new()
                    }
                )
                .unwrap(),
                RenderConfig::new()
            ),
            "\\nosuchcommand"
        );
    }

    #[test]
    fn t8() {
        assert_eq!(
            render(
                &parse("\\color{#abc}x", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "x"
        );
    }

    #[test]
    fn t9() {
        assert_eq!(
            render(
                &parse("\\verb|x|", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "x"
        );
    }

    #[test]
    fn t10() {
        assert_eq!(
            render(
                &parse("\\hbox{ab}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "ab"
        );
    }

    #[test]
    fn t11() {
        assert_eq!(
            render(
                &parse("\\big( x \\big)", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "(x)"
        );
        assert_eq!(
            render(
                &parse("\\bigl[ x \\bigr]", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "[x]"
        );
        assert_eq!(
            render(
                &parse("\\Bigg\\{ x \\Bigg\\}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "{x}"
        );
    }

    #[test]
    fn t12() {
        assert_eq!(
            render(
                &parse("\\left( x \\middle| y \\right)", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "(x ∣ y)"
        );
    }

    #[test]
    fn t13() {
        assert_eq!(
            render(
                &parse("a \\mathbin{+} b", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "a + b"
        );
        assert_eq!(
            render(
                &parse("a \\mathrel{=} b", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "a = b"
        );
        assert_eq!(
            render(
                &parse("\\mathop{x}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "x"
        );
        assert_eq!(
            render(
                &parse("\\mathopen{(}a\\mathclose{)}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "(a)"
        );
        assert_eq!(
            render(
                &parse("\\mathpunct{,}b", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            ", b"
        );
        assert_eq!(
            render(
                &parse("\\mathinner{a}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "a"
        );
    }

    #[test]
    fn t14() {
        assert_eq!(
            render(
                &parse("\\textcircled{a}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "textcircled(a)"
        );
        assert_eq!(
            render(
                &parse("\\mathstrut", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "vphantom(()"
        );
    }
}
