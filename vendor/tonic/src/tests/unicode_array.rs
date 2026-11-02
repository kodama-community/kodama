// Ported from moonbit inspect/unicode_array_test.mbt
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
                    "\\begin{cases} a & \\text{if } b \\\\ \\\\ c & \\text{if } d\\end{cases}",
                    &mut settings
                )
                .unwrap(),
                RenderConfig::new()
            ),
            "{a  if b\n{       \n{c  if d"
        );
        assert_eq!(
            render(
                &parse("\\left\\{ x \\right.", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            "{x"
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
                &parse("\\begin{dcases}a & b \\end{dcases}", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            "{a  b"
        );
        assert_eq!(
            render(
                &parse("\\begin{rcases}a & b \\end{rcases}", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            "a  b}"
        );
        assert_eq!(
            render(
                &parse("\\begin{drcases}a & b \\end{drcases}", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            "a  b}"
        );
    }

    #[test]
    fn t2() {
        assert_eq!(
            render(
                &parse(
                    "\\begin{array}{cc}a & b \\\\ c & d\\end{array}",
                    &mut Settings::new()
                )
                .unwrap(),
                RenderConfig::new()
            ),
            "a, b; c, d"
        );
    }

    #[test]
    fn t3() {
        let mut settings = Settings {
            display_mode: true,
            ..Settings::new()
        };
        let config = RenderConfig {
            line_style: LineStyle::Ascii,
            ..RenderConfig::new()
        };
        assert_eq!(
            render(
                &parse(
                    "\\begin{array}{|c|c|} \\hline a & b \\\\ \\hline c & d\\end{array}",
                    &mut settings
                )
                .unwrap(),
                RenderConfig::new()
            ),
            "┌──┬──┐\n│a │ b│\n├──┼──┤\n│c │ d│"
        );
        assert_eq!(
            render(
                &parse(
                    "\\begin{array}{|c|c|} \\hline a & b \\\\ \\hline c & d\\end{array}",
                    &mut settings
                )
                .unwrap(),
                config.clone()
            ),
            "+--+--+\n|a | b|\n+--+--+\n|c | d|"
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
                    "\\begin{array}{|c|c|} a & b \\\\ \\hline c & d \\\\ \\hline\\end{array}",
                    &mut settings
                )
                .unwrap(),
                RenderConfig::new()
            ),
            "│a │ b│\n├──┼──┤\n│c │ d│\n└──┴──┘"
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
                    "\\begin{array}{|c:c|} \\hdashline a & b \\\\ \\hdashline c & d\\end{array}",
                    &mut settings
                )
                .unwrap(),
                RenderConfig::new()
            ),
            "┌┄┄┬┄┄┐\n│a ┊ b│\n├┄┄┼┄┄┤\n│c ┊ d│"
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
                &parse(
                    "\\begin{pmatrix}𝟙 & x \\\\ aa & yy\\end{pmatrix}",
                    &mut settings
                )
                .unwrap(),
                RenderConfig::new()
            ),
            "(𝟙   x )\n(aa  yy)"
        );
        assert_eq!(
            render(
                &parse(
                    "\\begin{pmatrix}\\text{中} & b \\\\ aa & c\\end{pmatrix}",
                    &mut settings
                )
                .unwrap(),
                RenderConfig::new()
            ),
            "(中  b)\n(aa  c)"
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
                &parse(
                    "\\begin{array}{|c|} \\hline a \\\\ b \\\\ \\hline\\end{array}",
                    &mut settings
                )
                .unwrap(),
                RenderConfig::new()
            ),
            "┌─┐\n│a│\n│b│\n└─┘"
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
                &parse(
                    "\\begin{array}{|c|c|} \\hline a & bb \\\\ c \\end{array}",
                    &mut settings
                )
                .unwrap(),
                RenderConfig::new()
            ),
            "┌──┬───┐\n│a │ bb│\n│c │   │"
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
                &parse(
                    "\\begin{subarray}{l} a \\\\ b \\end{subarray}",
                    &mut settings
                )
                .unwrap(),
                RenderConfig::new()
            ),
            "a\nb"
        );
        assert_eq!(
            render(
                &parse("\\substack{a \\\\ b}", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            "a\nb"
        );
    }

    #[test]
    fn t10() {
        let mut settings = Settings {
            display_mode: true,
            ..Settings::new()
        };
        assert_eq!(render(&parse("x = \\begin{cases}\n   a &\\text{if } b \\\\ \\\\\n   c &\\text{if } d\n\\end{cases}", &mut settings).unwrap(), RenderConfig::new()), "    {a  if b\nx = {       \n    {c  if d");
        assert_eq!(
            render(
                &parse(
                    "y = \\begin{cases} x & p \\\\ z & q \\\\ w & r \\end{cases}",
                    &mut settings
                )
                .unwrap(),
                RenderConfig::new()
            ),
            "    {x  p\ny = {z  q\n    {w  r"
        );
    }
}
