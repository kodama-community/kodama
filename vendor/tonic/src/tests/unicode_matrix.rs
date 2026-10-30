// Ported from moonbit inspect/unicode_matrix_test.mbt
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
                &parse(
                    "\\begin{pmatrix}a & b \\\\ c & d\\end{pmatrix}",
                    &mut Settings::new()
                )
                .unwrap(),
                RenderConfig::new()
            ),
            "(a, b; c, d)"
        );
        assert_eq!(
            render(
                &parse(
                    "\\begin{bmatrix}a & b \\\\ c & d\\end{bmatrix}",
                    &mut Settings::new()
                )
                .unwrap(),
                RenderConfig::new()
            ),
            "[a, b; c, d]"
        );
        assert_eq!(
            render(
                &parse("\\begin{matrix}a \\\\ b\\end{matrix}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "a; b"
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
                    "\\begin{pmatrix}a & b \\\\ c & d\\end{pmatrix}",
                    &mut settings
                )
                .unwrap(),
                RenderConfig::new()
            ),
            "(a  b)\n(c  d)"
        );
        assert_eq!(
            render(
                &parse(
                    "\\begin{bmatrix}a & b \\\\ c & d\\end{bmatrix}",
                    &mut settings
                )
                .unwrap(),
                RenderConfig::new()
            ),
            "[a  b]\n[c  d]"
        );
        assert_eq!(
            render(
                &parse("\\begin{matrix}a \\\\ b\\end{matrix}", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            "a\nb"
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
                    "\\begin{pmatrix}aa & b \\\\ c & ddd\\end{pmatrix}",
                    &mut settings
                )
                .unwrap(),
                RenderConfig::new()
            ),
            "(aa  b  )\n(c   ddd)"
        );
        assert_eq!(
            render(
                &parse(
                    "\\begin{pmatrix}a & b \\\\ c & d\\end{pmatrix}",
                    &mut settings
                )
                .unwrap(),
                RenderConfig::new()
            ),
            "(a  b)\n(c  d)"
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
                    "\\begin{pmatrix}a \\\\ b \\\\ c\\end{pmatrix}",
                    &mut settings
                )
                .unwrap(),
                RenderConfig::new()
            ),
            "(a)\n(b)\n(c)"
        );
        assert_eq!(
            render(
                &parse("\\begin{pmatrix}aa \\\\ b\\end{pmatrix}", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            "(aa)\n(b )"
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
                &parse("\\begin{matrix} a \\\\ b \\\\ \\end{matrix}", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            "a\nb"
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
                    "\\begin{vmatrix}a & b \\\\ c & d\\end{vmatrix}",
                    &mut settings
                )
                .unwrap(),
                RenderConfig::new()
            ),
            "∣a  b∣\n∣c  d∣"
        );
        assert_eq!(
            render(
                &parse(
                    "\\begin{Vmatrix}a & b \\\\ c & d\\end{Vmatrix}",
                    &mut settings
                )
                .unwrap(),
                RenderConfig::new()
            ),
            "‖a  b‖\n‖c  d‖"
        );
        assert_eq!(
            render(
                &parse(
                    "\\begin{Bmatrix}a & b \\\\ c & d\\end{Bmatrix}",
                    &mut settings
                )
                .unwrap(),
                RenderConfig::new()
            ),
            "{a  b}\n{c  d}"
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
                    "\\begin{matrix*}[r]a & b \\\\ cc & d\\end{matrix*}",
                    &mut settings
                )
                .unwrap(),
                RenderConfig::new()
            ),
            "a   b\ncc  d"
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
                    "\\begin{smallmatrix}a & b \\\\ c & d\\end{smallmatrix}",
                    &mut settings
                )
                .unwrap(),
                RenderConfig::new()
            ),
            "a  b\nc  d"
        );
    }
}
