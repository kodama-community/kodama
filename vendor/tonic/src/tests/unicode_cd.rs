// Ported from moonbit inspect/unicode_cd_test.mbt
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
                    "\\begin{CD}\n   A @>a>> B \\\\\n@VbVV @AAcA \\\\\n   C @= D\n\\end{CD}",
                    &mut settings
                )
                .unwrap(),
                RenderConfig::new()
            ),
            "    a     \n A ───→ B \n │      ↑ \nb│      │c\n ↓      │ \n C ==== D "
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
                &parse("\\begin{CD} A @>a>> B \\end{CD}", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            "   a    \nA ───→ B"
        );
    }

    #[test]
    fn t2() {
        let mut settings = Settings {
            display_mode: true,
            ..Settings::new()
        };
        assert_eq!(render(&parse("\\begin{CD}\n   A @>a>> B @>c>> C \\\\\n@VdVV @VeVV @AAfA \\\\\n   D @= E @= F\n\\end{CD}", &mut settings).unwrap(), RenderConfig::new()), "     a      c     \n A ────→ B ───→ C \n │       │      ↑ \nd│      e│      │f\n ↓       ↓      │ \n D ===== E ==== F ");
    }

    #[test]
    fn t3() {
        let mut settings = Settings {
            display_mode: true,
            ..Settings::new()
        };
        assert_eq!(
            render(
                &parse("\\begin{CD} A @>>b>B \\end{CD}", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            "A ───→ B\n   b    "
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
                &parse("\\begin{CD} A @>a>b>B \\end{CD}", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            "   a    \nA ───→ B\n   b    "
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
                &parse("\\begin{CD} A @<a<< B \\end{CD}", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            "   a    \nA ←─── B"
        );
        assert_eq!(
            render(
                &parse("\\begin{CD} A @| B \\end{CD}", &mut settings).unwrap(),
                RenderConfig::new()
            ),
            "A ‖    B"
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
                    "\\begin{CD} A @>a>> B \\\\ @VbVcV @AAeAfA \\\\ C @= D \\end{CD}",
                    &mut settings
                )
                .unwrap(),
                RenderConfig::new()
            ),
            "     a     \n A  ───→ B \n │       ↑ \nb│c      │e\n ↓       │ \n C  ==== D "
        );
    }

    #[test]
    fn t7() {
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
                    "\\begin{CD} A @>a>> B \\\\ @VbVV @AAcA \\\\ C @= D \\end{CD}",
                    &mut settings
                )
                .unwrap(),
                config.clone()
            ),
            "    a     \n A ---→ B \n |      ↑ \nb|      |c\n ↓      | \n C ==== D "
        );
    }
}
