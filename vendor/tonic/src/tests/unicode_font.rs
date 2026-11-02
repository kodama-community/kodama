// Ported from moonbit inspect/unicode_font_test.mbt
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
                &parse("\\R_1", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "ℝ₁"
        );
        assert_eq!(
            render(
                &parse("\\R^2", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "ℝ²"
        );
        assert_eq!(
            render(
                &parse("\\R_1^2", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "ℝ₁²"
        );
        assert_eq!(
            render(
                &parse("\\frac{\\R}{2}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "ℝ∕2"
        );
        assert_eq!(
            render(
                &parse("\\frac{\\mathbb{abc}}{2}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "𝕒𝕓𝕔∕2"
        );
    }

    #[test]
    fn t1() {
        assert_eq!(
            render(
                &parse("\\mathbf{ab}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "𝐚𝐛"
        );
        assert_eq!(
            render(
                &parse("\\mathbf{abc}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "𝐚𝐛𝐜"
        );
        assert_eq!(
            render(
                &parse("\\mathbb{R}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "ℝ"
        );
        assert_eq!(
            render(
                &parse("\\mathbf{a\\alpha b}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "𝐚𝛂𝐛"
        );
        assert_eq!(
            render(
                &parse("\\mathbf{\\alpha\\beta}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "𝛂𝛃"
        );
        assert_eq!(
            render(
                &parse("\\mathbf{\\Gamma\\Omega}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "𝚪𝛀"
        );
        assert_eq!(
            render(
                &parse("\\boldsymbol{\\alpha}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "𝜶"
        );
        assert_eq!(
            render(
                &parse("\\mathbf{x+y}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "𝐱+𝐲"
        );
        assert_eq!(
            render(
                &parse("\\mathbf{x=y}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "𝐱=𝐲"
        );
    }
}
