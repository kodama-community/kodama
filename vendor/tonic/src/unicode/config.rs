use std::rc::Rc;

use crate::anvil::SpacingSpec;
use crate::ast::StyleLevel;

/// Configuration for the Unicode renderer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderConfig {
    pub fraction_slash: String,
    pub spacing: SpacingSpec,
    pub line_style: LineStyle,
}

/// Glyph style for drawn lines: fraction bars, boxes, tables, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LineStyle {
    Ascii,
    #[default]
    Unicode,
}

/// Unicode text spacing: regular spaces for thick/medium, thin and after
/// operators.
pub fn unicode_text_spacing() -> SpacingSpec {
    SpacingSpec {
        thick: " ".to_string(),
        medium: " ".to_string(),
        thin: " ".to_string(),
        operator: " ".to_string(),
    }
}

impl RenderConfig {
    pub fn new() -> Self {
        RenderConfig {
            fraction_slash: "∕".to_string(),
            spacing: unicode_text_spacing(),
            line_style: LineStyle::Unicode,
        }
    }
}

impl Default for RenderConfig {
    fn default() -> Self {
        RenderConfig::new()
    }
}

/// The line character for a fraction bar under the given style.
pub fn line_style_frac_bar(style: LineStyle) -> String {
    match style {
        LineStyle::Ascii => "-".to_string(),
        LineStyle::Unicode => "─".to_string(),
    }
}

#[derive(Clone)]
pub(crate) struct RenderState {
    pub style: StyleLevel,
    pub in_display: bool,
    pub in_tight: bool,
    pub config: Rc<RenderConfig>,
}
