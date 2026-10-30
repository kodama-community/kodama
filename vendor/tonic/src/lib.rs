//! A Rust port of the KaTeX parser: lexes and parses LaTeX math expressions
//! with macro expansion into a typed [`ParseNode`] AST, and renders that AST
//! to Unicode text or, with the `html` feature, KaTeX-compatible HTML.

mod anvil;
mod ast;
mod builtin_macros;
mod builtin_macros_commands;
mod builtin_macros_control;
mod builtin_macros_dots;
mod builtin_macros_special;
mod builtin_macros_static;
mod environments;
mod error;
mod function_registry;
mod functions;
#[cfg(feature = "html")]
pub mod html;
mod lexer;
mod macro_definition;
mod macro_expander;
mod namespace;
mod parser;
mod settings;
mod source_location;
mod symbol_registry;
mod text_ligature;
mod token;
mod unicode;
mod unicode_font;
mod unicode_scripts;
mod unicode_symbols;

pub use anvil::{
    SpacableItem, SpacingSpec, atom_family_name, cancel_bin_atoms, command_name, em_value,
    is_null_delimiter, join_with_spacing, katex_size_multiplier, math_choice_variant, math_spacing,
    resolve_symbol,
};
pub use ast::{
    ArrayColumn, AtomFamily, ColumnSeparationType, LapAlignment, Measurement, Mode,
    OperatorContent, ParseNode, StyleLevel,
};
pub use environments::{
    ArrayEnvironmentOptions, EnvironmentContext, EnvironmentHandler, EnvironmentParser,
    EnvironmentRegistry, EnvironmentSpec, build_environment_registry,
};
pub use error::{Diagnostic, ParseError};
pub use function_registry::{
    ArgType, FunctionContext, FunctionHandler, FunctionParser, FunctionRegistry, FunctionSpec,
    build_function_registry,
};
#[cfg(feature = "html")]
pub use html::{render as render_html, RenderConfig as HtmlRenderConfig};
pub use macro_definition::{MacroDefinition, MacroExpansion};
pub use parser::{parse, parse_with_specs};
pub use settings::{
    Macros, Settings, StrictHandler, StrictResponse, StrictWarningHandler, Strictness,
    TrustContext, TrustHandler, TrustPolicy,
};
pub use source_location::SourceLocation;
pub use symbol_registry::unicode_symbol;
pub use token::{Token, token_location};
pub use unicode::{
    Block, LineStyle, RenderConfig, line_style_frac_bar, render, unicode_text_spacing,
};
pub use unicode_font::unicode_font_character;
pub use unicode_scripts::{UnicodeScriptKind, supported_codepoint, unicode_script_character};

#[cfg(test)]
mod tests;
