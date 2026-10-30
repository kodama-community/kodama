// Ported from moonbit inspect/unicode_operator_test.mbt
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
                &parse("\\det A", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "det A"
        );
        assert_eq!(
            render(
                &parse("\\sin x", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "sin x"
        );
        assert_eq!(
            render(
                &parse("\\lim f", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "lim f"
        );
        assert_eq!(
            render(
                &parse("\\det A_1", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "det A₁"
        );
        assert_eq!(
            render(
                &parse("\\det(x)", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "det(x)"
        );
        assert_eq!(
            render(
                &parse("\\det\\left(x\\right)", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "det (x)"
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
                &parse("\\sum_{i=0}^n x", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            " n   \n ∑  x\ni=0  "
        );
        assert_eq!(
            render(
                &parse("\\lim_{x\\to 0} f", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            "lim f\nx→0  "
        );
        assert_eq!(
            render(
                &parse("\\int_0^1 x", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            "∫₀¹ x"
        );
        assert_eq!(
            render(
                &parse("\\sum_{i=0}^n x", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "∑ᵢ₌₀ⁿ x"
        );
    }

    #[test]
    fn t2() {
        assert_eq!(
            render(
                &parse("\\operatorname*{argmin}_{x} f", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "argmin f\n  x     "
        );
        assert_eq!(
            render(
                &parse("\\operatorname{argmin}_{x} f", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "argminₓ f"
        );
    }

    #[test]
    fn t3() {
        assert_eq!(
            render(
                &parse("\\sin", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "sin"
        );
        assert_eq!(
            render(
                &parse("\\lim_{x}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "limₓ"
        );
        assert_eq!(
            render(
                &parse("\\det A", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "det A"
        );
        assert_eq!(
            render(
                &parse("\\widehat{\\overrightarrow{x}}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "widehat(overrightarrow(x))"
        );
        assert_eq!(
            render(
                &parse("\\hat{\\cancel{x}}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "hat(cancel(x))"
        );
        assert_eq!(
            render(
                &parse("\\underleftrightarrow{x}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "underaccent(underleftrightarrow,x)"
        );
    }
}
