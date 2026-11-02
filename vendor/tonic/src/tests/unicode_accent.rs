// Ported from moonbit inspect/unicode_accent_test.mbt
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
                &parse("\\hat{x}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "x̂"
        );
        assert_eq!(
            render(
                &parse("\\hat{x+y}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "hat(x + y)"
        );
    }

    #[test]
    fn t1() {
        assert_eq!(
            render(
                &parse("\\acute{x}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "x́"
        );
        assert_eq!(
            render(
                &parse("\\grave{x}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "x̀"
        );
        assert_eq!(
            render(
                &parse("\\ddot{x}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "ẍ"
        );
        assert_eq!(
            render(
                &parse("\\tilde{x}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "x̃"
        );
        assert_eq!(
            render(
                &parse("\\bar{x}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "x̄"
        );
        assert_eq!(
            render(
                &parse("\\breve{x}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "x̆"
        );
        assert_eq!(
            render(
                &parse("\\check{x}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "x̌"
        );
        assert_eq!(
            render(
                &parse("\\dot{x}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "ẋ"
        );
        assert_eq!(
            render(
                &parse("\\mathring{x}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "x̊"
        );
        assert_eq!(
            render(
                &parse("\\vec{x}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "x⃗"
        );
    }

    #[test]
    fn t2() {
        assert_eq!(
            render(
                &parse("\\hat{\\overline{x}}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "hat(overline(x))"
        );
        assert_eq!(
            render(
                &parse("\\hat{\\left(x\\right)}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "hat((x))"
        );
        assert_eq!(
            render(
                &parse("\\hat{x}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "x̂"
        );
    }
}
