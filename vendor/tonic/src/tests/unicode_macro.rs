// Ported from moonbit inspect/unicode_macro_test.mbt
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
        let mut settings = Settings::new();
        settings.global_group = true;
        settings.macro_store = Some(Macros::new(HashMap::new()));
        parse("\n\\def\\thm{\\@ifnextchar[{\\thm@with}{\\thm@without}}\n\\def\\thm@with[#1]{\\textbf{Theorem~(#1).}}\n\\def\\thm@without{\\textbf{Theorem.}}", &mut settings).unwrap();
        let nodes = parse("\\thm[Hirzebruch--Riemann--Roch]", &mut settings).unwrap();
        assert_eq!(
            format!("\"{}\"", render(&nodes, RenderConfig::new())),
            "\"Theorem (Hirzebruch–Riemann–Roch).\""
        );
    }

    #[test]
    fn t1() {
        assert_eq!(
            render(
                &parse("\\def\\foo{x} \\foo", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "x"
        );
    }

    #[test]
    fn t2() {
        assert_eq!(
            render(
                &parse("\\gdef\\foo{x} \\foo", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "x"
        );
    }

    #[test]
    fn t3() {
        assert_eq!(
            render(
                &parse("\\newcommand{\\foo}{x} \\foo", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "x"
        );
    }

    #[test]
    fn t4() {
        assert_eq!(
            render(
                &parse(
                    "\\def\\vect#1{\\mathbf{#1}} \\vect{x}",
                    &mut Settings::new()
                )
                .unwrap(),
                RenderConfig::new()
            ),
            "𝐱"
        );
    }

    #[test]
    fn t5() {
        assert_eq!(
            render(
                &parse(
                    "\\def\\div#1#2{\\frac{#1}{#2}} \\div{a}{b}",
                    &mut Settings::new()
                )
                .unwrap(),
                RenderConfig::new()
            ),
            "a∕b"
        );
    }

    #[test]
    fn t6() {
        assert_eq!(
            render(
                &parse("\\def\\half{\\frac{1}{2}} \\half", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "1∕2"
        );
    }

    #[test]
    fn t7() {
        assert_eq!(
            render(
                &parse("\\def\\R{\\mathbb{R}} \\R_1^2", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "ℝ₁²"
        );
    }

    #[test]
    fn t8() {
        assert_eq!(
            render(
                &parse(
                    "\\def\\foo{a} \\foo \\def\\foo{b} \\foo",
                    &mut Settings::new()
                )
                .unwrap(),
                RenderConfig::new()
            ),
            "ab"
        );
    }

    #[test]
    fn t9() {
        assert_eq!(
            render(
                &parse("\\def\\expr{a+b} \\expr", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "a + b"
        );
    }

    #[test]
    fn t10() {
        let macros = HashMap::from([("\\foo".to_string(), "x".to_string())]);
        let mut settings = Settings {
            macros,
            ..Settings::new()
        };
        assert_eq!(
            render(&parse("\\foo", &mut settings).unwrap(), RenderConfig::new()),
            "x"
        );
    }

    #[test]
    fn t11() {
        let macros = HashMap::from([("\\vect".to_string(), "\\mathbf{#1}".to_string())]);
        let mut settings = Settings {
            macros,
            ..Settings::new()
        };
        assert_eq!(
            render(
                &parse("\\vect{x}", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            "𝐱"
        );
    }

    #[test]
    fn t12() {
        let macros = HashMap::from([("\\R".to_string(), "\\mathbb{R}".to_string())]);
        let mut settings = Settings {
            macros,
            ..Settings::new()
        };
        assert_eq!(
            render(
                &parse("\\R_1^2", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            "ℝ₁²"
        );
    }
}
