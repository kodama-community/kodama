// Ported from moonbit inspect/unicode_align_test.mbt
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
                    "\\begin{aligned}a &= b \\\\ cccc &= d\\end{aligned}",
                    &mut settings
                )
                .unwrap(),
                RenderConfig::new()
            ),
            "   a = b\ncccc = d"
        );
        assert_eq!(
            render(
                &parse(
                    "\\begin{aligned}\\log_{\\phi}\\sqrt5-1 \\\\ &= 1\\end{aligned}",
                    &mut settings
                )
                .unwrap(),
                RenderConfig::new()
            ),
            "logᵩ √5 − 1    \n            = 1"
        );
        assert_eq!(
            render(
                &parse(
                    "\\begin{split}a &= b \\\\ c &= d\\end{split}",
                    &mut settings
                )
                .unwrap(),
                RenderConfig::new()
            ),
            "a = b\nc = d"
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
                &parse(
                    "\\begin{aligned}a &= b & c &= d \\\\ eeee &= f & g &= h\\end{aligned}",
                    &mut settings
                )
                .unwrap(),
                RenderConfig::new()
            ),
            "   a = b  c = d\neeee = f  g = h"
        );
        assert_eq!(
            render(
                &parse(
                    "\\begin{alignedat}{2}a &= b & c &= d \\\\ ee &= f & gg &= h\\end{alignedat}",
                    &mut settings
                )
                .unwrap(),
                RenderConfig::new()
            ),
            " a = b c = d\nee = fgg = h"
        );
    }

    #[test]
    fn t2() {
        let mut settings = Settings {
            display_mode: true,
            ..Settings::new()
        };
        assert_eq!(
            render(
                &parse(
                    "\\begin{aligned}x &= \\dfrac{1}{2} \\\\ y &= z\\end{aligned}",
                    &mut settings
                )
                .unwrap(),
                RenderConfig::new()
            ),
            "     1 \nx = ───\n     2 \ny = z  "
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
                &parse(
                    "\\begin{gathered}x \\\\ y + 2\\end{gathered}",
                    &mut settings
                )
                .unwrap(),
                RenderConfig::new()
            ),
            "      x      \n    y + 2    "
        );
        assert_eq!(
            render(
                &parse(
                    "\\begin{gathered}x \\\\ yyyyy\\end{gathered}",
                    &mut settings
                )
                .unwrap(),
                RenderConfig::new()
            ),
            "      x      \n    yyyyy    "
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
                &parse(
                    "\\begin{align}a &= b \\\\ c &= d\\end{align}",
                    &mut settings
                )
                .unwrap(),
                RenderConfig::new()
            ),
            "a = b\nc = d"
        );
        assert_eq!(
            render(
                &parse(
                    "\\begin{align*}a &= b \\\\ c &= d\\end{align*}",
                    &mut settings
                )
                .unwrap(),
                RenderConfig::new()
            ),
            "a = b\nc = d"
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
                &parse(
                    "\\begin{alignat}{2}a &= b & c &= d \\\\ e &= f & g &= h\\end{alignat}",
                    &mut settings
                )
                .unwrap(),
                RenderConfig::new()
            ),
            "a = bc = d\ne = fg = h"
        );
        assert_eq!(
            render(
                &parse(
                    "\\begin{alignat*}{2}a &= b & c &= d \\\\ e &= f & g &= h\\end{alignat*}",
                    &mut settings
                )
                .unwrap(),
                RenderConfig::new()
            ),
            "a = bc = d\ne = fg = h"
        );
    }

    #[test]
    fn t6() {
        let mut settings = Settings {
            display_mode: true,
            ..Settings::new()
        };
        assert_eq!(
            render(
                &parse("\\begin{gather}a \\\\ b\\end{gather}", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            "    a    \n    b    "
        );
        assert_eq!(
            render(
                &parse("\\begin{gather*}a \\\\ b\\end{gather*}", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            "    a    \n    b    "
        );
        assert_eq!(
            render(
                &parse("\\begin{equation}x\\end{equation}", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            "x"
        );
        assert_eq!(
            render(
                &parse("\\begin{equation*}x\\end{equation*}", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            "x"
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
                &parse("\\tag{a} x", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            "x\t(a)"
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
                &parse("\\overset{ab}{c}", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            "ab\nc "
        );
        assert_eq!(
            render(
                &parse("\\underset{ab}{c}", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            "c \nab"
        );
    }
}
