//! Shared, backend-agnostic helpers for the rendering backends (mathml,
//! typst, unicode). These encode KaTeX facts that each backend would otherwise
//! re-implement with drift risk: symbol resolution, null delimiters, control
//! sequence names, font sizes, unit conversions, and style selection.

use crate::ast::{Measurement, ParseNode, StyleLevel};
use crate::symbol_registry::unicode_symbol;

/// Resolves a TeX command name or raw Unicode string to its Unicode rendering
/// via the parser's symbol registry, falling back to the input itself.
pub fn resolve_symbol(text: &str) -> String {
    unicode_symbol(text).unwrap_or_else(|| text.to_string())
}

/// True for the null delimiter `.` used by `\left.` / `\right.`, which
/// renders as nothing in every backend.
pub fn is_null_delimiter(text: &str) -> bool {
    text == "."
}

/// Strips the leading backslash from a control-sequence label (`\sin` ->
/// `sin`). Labels without a backslash are returned unchanged.
pub fn command_name(label: &str) -> String {
    label.strip_prefix('\\').unwrap_or(label).to_string()
}

/// The KaTeX font-size multiplier table (`\tiny` .. `\Huge`, including
/// `\sixptsize`), indexed by the 1-based `Sizing.size` field. Sizes outside
/// the table fall back to 1.0 (`\normalsize`).
pub fn katex_size_multiplier(size: usize) -> f64 {
    const SIZES: [f64; 11] = [0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.2, 1.44, 1.728, 2.074, 2.488];
    size.checked_sub(1)
        .and_then(|index| SIZES.get(index))
        .copied()
        .unwrap_or(1.0)
}

/// Converts a measurement to an em length: `em` passes through, `mu` uses
/// the 18 mu per em ratio, and `ex` the standard 0.5 x-height ratio. Returns
/// `None` for units with no fixed em relation (`fill`, `pt`, ...).
pub fn em_value(measurement: &Measurement) -> Option<f64> {
    match measurement.unit.as_str() {
        "em" => Some(measurement.number),
        "mu" => Some(measurement.number / 18.0),
        "ex" => Some(measurement.number * 0.5),
        _ => None,
    }
}

/// Selects the body of a `\mathchoice` node for the current style level.
pub fn math_choice_variant(
    display: &[ParseNode],
    text: &[ParseNode],
    script: &[ParseNode],
    scriptscript: &[ParseNode],
    style: StyleLevel,
) -> Vec<ParseNode> {
    match style {
        StyleLevel::DisplayStyle => display.to_vec(),
        StyleLevel::TextStyle => text.to_vec(),
        StyleLevel::ScriptStyle => script.to_vec(),
        StyleLevel::ScriptScriptStyle => scriptscript.to_vec(),
    }
}
