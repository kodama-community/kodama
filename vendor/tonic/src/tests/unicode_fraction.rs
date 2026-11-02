// Ported from moonbit inspect/unicode_fraction_test.mbt
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
                    "\\frac{p}{q}=a_0+\\cfrac{1}{a_1+\\cfrac{1}{a_2+\\cfrac{1}{a_3}}}",
                    &mut settings
                )
                .unwrap(),
                RenderConfig::new()
            ),
            " p                 1         \n─── = a₀ + ──────────────────\n q                    1      \n            a₁ + ─────────── \n                        1    \n                  a₂ + ────  \n                        a₃   "
        );
    }

    #[test]
    fn t1() {
        assert_eq!(
            render(
                &parse("\\binom{n}{k}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "(n,k)"
        );
    }

    #[test]
    fn t2() {
        assert_eq!(
            render(
                &parse("\\frac{\\frac{a}{b}}{c}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "(a∕b)∕c"
        );
        assert_eq!(
            render(
                &parse("\\frac{a}{\\frac{b}{c}}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "a∕(b∕c)"
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
    fn t3() {
        let mut settings = Settings {
            display_mode: true,
            ..Settings::new()
        };
        assert_eq!(
            render(
                &parse("\\frac{a}{b}", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            " a \n───\n b "
        );
        assert_eq!(
            render(
                &parse("\\frac{a}{bc}", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            " a  \n────\n bc "
        );
        assert_eq!(
            render(
                &parse("\\frac{xy}{z}", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            " xy \n────\n z  "
        );
        assert_eq!(
            render(
                &parse("\\frac{a^2}{b+c}", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            "  a²   \n───────\n b + c "
        );
        assert_eq!(
            render(
                &parse("\\dfrac{x}{y}", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            " x \n───\n y "
        );
        assert_eq!(
            render(
                &parse("\\binom{n}{k}", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            "(n)\n(k)"
        );
    }

    #[test]
    fn t4() {
        let mut settings = Settings {
            display_mode: true,
            ..Settings::new()
        };
        assert_eq!(
            render(
                &parse("\\left(\\frac{1}{2}\\right)", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            "  1  \n(───)\n  2  "
        );
        assert_eq!(
            render(
                &parse("\\left[\\frac{a}{bc}\\right]", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            "  a   \n[────]\n  bc  "
        );
        assert_eq!(
            render(
                &parse("\\left(\\frac{x^2}{y+z}\\right)", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            "   x²    \n(───────)\n  y + z  "
        );
        assert_eq!(
            render(
                &parse("\\genfrac{(}{)}{0.4pt}{1}{x}{y}", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            "  x  \n(───)\n  y  "
        );
    }

    #[test]
    fn t5() {
        let mut settings = Settings {
            display_mode: true,
            ..Settings::new()
        };
        assert_eq!(
            render(
                &parse("a+\\left(\\frac{1}{2}\\right)", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            "      1  \na + (───)\n      2  "
        );
    }

    #[test]
    fn t6() {
        assert_eq!(
            render(
                &parse("\\left(\\frac{1}{2}\\right)", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "(1∕2)"
        );
        assert_eq!(
            render(
                &parse("\\left( x \\right)", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "(x)"
        );
    }

    #[test]
    fn t7() {
        let mut settings = Settings {
            display_mode: true,
            ..Settings::new()
        };
        assert_eq!(
            render(
                &parse("\\frac{aaaaaaa}{b}", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            " aaaaaaa \n─────────\n    b    "
        );
        assert_eq!(
            render(
                &parse("\\frac{a}{bbbbbbb}", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            "    a    \n─────────\n bbbbbbb "
        );
        assert_eq!(
            render(
                &parse("\\frac{aaa}{bbb}", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            " aaa \n─────\n bbb "
        );
    }

    #[test]
    fn t8() {
        let config = RenderConfig {
            fraction_slash: "/".to_string(),
            ..RenderConfig::new()
        };
        assert_eq!(
            render(
                &parse("\\frac{a}{b}", &mut Settings::new()).unwrap(),
                config.clone()
            ),
            "a/b"
        );
        assert_eq!(
            render(
                &parse("\\frac{a+b}{c+d}", &mut Settings::new()).unwrap(),
                config.clone()
            ),
            "(a+b)/(c+d)"
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
                &parse("\\left({\\frac{a}{b}}\\right)", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            "  a  \n(───)\n  b  "
        );
        assert_eq!(
            render(
                &parse("\\left\\{\\frac{a}{b}\\right.", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            "  a \n{───\n  b "
        );
        assert_eq!(
            render(
                &parse("a{\\frac{b}{c}}d", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            "  b  \na───d\n  c  "
        );
        assert_eq!(
            render(
                &parse("\\frac{a}{b}_1", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            "  a   \n(───)₁\n  b   "
        );
        assert_eq!(
            render(
                &parse("a \\atop b", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            " a \n b "
        );
        assert_eq!(
            render(
                &parse("\\genfrac{}{}{0pt}{}{a}{b}", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            " a \n b "
        );
        assert_eq!(
            render(
                &parse("\\genfrac{}{}{0.4pt}{}{a}{b}", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            " a \n───\n b "
        );
        assert_eq!(
            render(
                &parse("\\sqrt{a \\atop b}", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            "   a  \n√( b )"
        );
        assert_eq!(
            render(
                &parse("a \\atop b", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "a,b"
        );
    }
}
