// Ported from moonbit inspect/unicode_scripts_test.mbt
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
                &parse("x^{12}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "x¹²"
        );
        assert_eq!(
            render(
                &parse("x_{ij}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "xᵢⱼ"
        );
    }

    #[test]
    fn t1() {
        assert_eq!(
            render(
                &parse("x^{a+b}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "xᵃ⁺ᵇ"
        );
        assert_eq!(
            render(
                &parse("x_{i+j}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "xᵢ₊ⱼ"
        );
        assert_eq!(
            render(
                &parse("x^2_n", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "xₙ²"
        );
        assert_eq!(
            render(
                &parse("x_i^j", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "xᵢʲ"
        );
    }
}
