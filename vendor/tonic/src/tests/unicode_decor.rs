// Ported from moonbit inspect/unicode_decor_test.mbt
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
                &parse("\\overbrace{a}^{b}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "overbrace(a)ᵇ"
        );
        assert_eq!(
            render(
                &parse("\\underbrace{a}_{b}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "underbrace(a)_b"
        );
        assert_eq!(
            render(
                &parse("\\overbracket{a}^{b}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "overbracket(a)ᵇ"
        );
        assert_eq!(
            render(
                &parse("\\underbracket{a}_{b}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "underbracket(a)_b"
        );
    }

    #[test]
    fn t1() {
        assert_eq!(
            render(
                &parse("\\xleftarrow{a}^{b}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "xleftarrow(a)ᵇ"
        );
        assert_eq!(
            render(
                &parse("\\xrightarrow{a}^{b}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "xrightarrow(a)ᵇ"
        );
        assert_eq!(
            render(
                &parse("\\xRightarrow{a}^{b}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "xRightarrow(a)ᵇ"
        );
        assert_eq!(
            render(
                &parse("\\xmapsto{a}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "xmapsto(a)"
        );
        assert_eq!(
            render(
                &parse("\\xrightarrow[below]{above}", &mut Settings::new()).unwrap(),
                RenderConfig::new()
            ),
            "xrightarrow(above,below)"
        );
    }
}
