use std::collections::HashMap;

use crate::ast::{Measurement, Mode, ParseNode};
use crate::error::ParseError;
use crate::macro_definition::MacroDefinition;
use crate::settings::TrustContext;
use crate::token::{token_location, Token};

use crate::functions::*;

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// How a function argument should be parsed.
pub enum ArgType {
    ColorArg,
    SizeArg,
    UrlArg,
    RawArg,
    OriginalArg,
    HboxArg,
    PrimitiveArg,
    MathArg,
    TextArg,
}

/// Callbacks the parser exposes to function handlers. `&mut` methods mutate
/// parser state; `&self` methods are pure reads of settings/state.
pub trait FunctionParser {
    fn report_nonstrict(
        &self,
        error_code: &str,
        error_message: &str,
        token: Option<&Token>,
    ) -> Result<(), ParseError>;
    fn use_strict_behavior(
        &self,
        error_code: &str,
        error_message: &str,
        token: Option<&Token>,
    ) -> bool;
    fn is_trusted(&self, context: TrustContext) -> bool;
    /// Format a rejected command as KaTeX's colored text. Implementations with
    /// a custom error color should override this method.
    fn unsupported_command(&self, mode: Mode, text: &str, fallback: ParseNode) -> ParseNode {
        crate::parser::unsupported_command(mode, "#cc0000", text, fallback)
    }
    fn current_color(&self) -> Result<Option<String>, ParseError>;
    fn in_left_right(&self) -> bool;
    fn is_expandable(&self, name: &str) -> bool;
    fn get_macro(&self, name: &str) -> Option<MacroDefinition>;
    fn set_macro(&mut self, name: &str, definition: Option<MacroDefinition>);
    fn set_macro_definition(&mut self, name: &str, definition: MacroDefinition, global: bool);
    fn parse_expression(
        &mut self,
        expr_list: bool,
        break_on_token_text: Option<&str>,
    ) -> Result<Vec<ParseNode>, ParseError>;
    fn parse_math_mode(&mut self, closing: &str) -> Result<Vec<ParseNode>, ParseError>;
    fn parse_left_right(&mut self, open: &str) -> Result<ParseNode, ParseError>;
    fn parse_optional_size(&mut self) -> Result<Option<Measurement>, ParseError>;
    fn parse_prefixed_function(&mut self, name: &str) -> Result<ParseNode, ParseError>;
    fn parse_environment(&mut self, name: &str) -> Result<ParseNode, ParseError>;
    fn pop_token(&mut self) -> Result<Token, ParseError>;
    fn future_token(&mut self) -> Result<Token, ParseError>;
    fn push_token(&mut self, token: Token);
    fn consume_spaces(&mut self) -> Result<(), ParseError>;
    fn consume_macro_arg(&mut self) -> Result<Vec<Token>, ParseError>;
    fn expand_tokens(&mut self, tokens: Vec<Token>) -> Result<Vec<Token>, ParseError>;
}

/// Per-function-call context: static data about the function being parsed.
#[derive(Debug, Clone)]
pub struct FunctionContext {
    pub func_name: String,
    pub mode: Mode,
    pub token: Option<Token>,
    pub break_on_token_text: Option<String>,
    pub display_mode: bool,
}

/// A function handler implementation.
pub type FunctionHandler = fn(
    parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError>;

#[derive(Debug, Clone)]
/// The declaration of a function (name, arguments, handler).
pub struct FunctionSpec {
    pub names: Vec<String>,
    pub num_args: usize,
    pub num_optional_args: usize,
    pub arg_types: Vec<ArgType>,
    pub allowed_in_argument: bool,
    pub allowed_in_text: bool,
    pub allowed_in_math: bool,
    pub infix: bool,
    pub primitive: bool,
    pub primitive_after_missing_optional: Option<usize>,
    pub handler: Option<FunctionHandler>,
}

impl Default for FunctionSpec {
    fn default() -> Self {
        FunctionSpec {
            names: Vec::new(),
            num_args: 0,
            num_optional_args: 0,
            arg_types: Vec::new(),
            allowed_in_argument: false,
            allowed_in_text: false,
            allowed_in_math: true,
            infix: false,
            primitive: false,
            primitive_after_missing_optional: None,
            handler: None,
        }
    }
}

impl FunctionSpec {
    pub fn is_expandable(&self) -> bool {
        !self.primitive
    }
}

#[derive(Clone, Default)]
/// A map from function names to their specs.
pub struct FunctionRegistry {
    entries: HashMap<String, FunctionSpec>,
}

impl FunctionRegistry {
    pub fn new() -> Self {
        FunctionRegistry {
            entries: HashMap::new(),
        }
    }

    pub fn register(&mut self, spec: FunctionSpec) {
        for name in &spec.names {
            self.entries.insert(name.clone(), spec.clone());
        }
    }

    pub fn get(&self, name: &str) -> Option<&FunctionSpec> {
        self.entries.get(name)
    }

    pub fn keys(&self) -> Vec<String> {
        self.entries.keys().cloned().collect()
    }
}

fn verb_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec!["\\verb".to_string()],
        allowed_in_text: true,
        handler: Some(verb_handler),
        ..Default::default()
    }
}

fn verb_handler(
    _parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    _args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    let loc = token_location(context.token.as_ref());
    Err(ParseError::InvalidArgument {
        message: "\\verb ended by end of line instead of matching delimiter".to_string(),
        loc,
    })
}

fn relax_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec!["\\relax".to_string()],
        allowed_in_argument: true,
        allowed_in_text: true,
        handler: Some(relax_handler),
        ..Default::default()
    }
}

fn relax_handler(
    _parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    _args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    Ok(ParseNode::Internal { mode: context.mode })
}

pub fn builtin_function_specs() -> Vec<FunctionSpec> {
    vec![
        verb_spec(),
        relax_spec(),
        sqrt_spec(),
        standard_genfrac_spec(),
        infix_genfrac_spec(),
        general_genfrac_spec(),
        above_spec(),
        abovefrac_spec(),
        text_spec(),
        textcolor_spec(),
        color_spec(),
        styling_spec(),
        font_spec(),
        boldsymbol_spec(),
        old_font_spec(),
        mclass_spec(),
        binrel_spec(),
        stackrel_spec(),
        big_operator_spec(),
        mathop_spec(),
        named_operator_spec(),
        limited_named_operator_spec(),
        integral_operator_spec(),
        operatorname_spec(),
        overline_spec(),
        underline_spec(),
        smash_spec(),
        phantom_spec(),
        vphantom_spec(),
        pmb_spec(),
        vcenter_spec(),
        rule_spec(),
        raisebox_spec(),
        reflectbox_spec(),
        mathreflectbox_spec(),
        hbox_spec(),
        lap_spec(),
        mathchoice_spec(),
        sizing_spec(),
        char_spec(),
        horiz_brace_spec(),
        x_arrow_spec(),
        accent_under_spec(),
        accent_spec(),
        text_accent_spec(),
        kern_spec(),
        colorbox_spec(),
        fcolorbox_spec(),
        fbox_spec(),
        cancel_spec(),
        sout_spec(),
        angl_spec(),
        href_spec(),
        url_spec(),
        html_spec(),
        cr_spec(),
        macro_prefix_spec(),
        definition_spec(),
        let_spec(),
        futurelet_spec(),
        includegraphics_spec(),
        begin_end_spec(),
        hline_spec(),
        cd_internal_spec(),
        cd_parent_spec(),
        html_mathml_spec(),
        math_mode_spec(),
        math_closing_spec(),
        delim_sizing_spec(),
        left_right_closing_spec(),
        left_right_spec(),
        middle_spec(),
    ]
}

/// Builds a function registry from the builtin specs plus caller-provided
/// extension specs (e.g. `\eval` registered by the `eval` package).
pub fn build_function_registry(extra_specs: &[FunctionSpec]) -> FunctionRegistry {
    let mut registry = FunctionRegistry::new();
    for spec in builtin_function_specs() {
        registry.register(spec);
    }
    for spec in extra_specs {
        registry.register(spec.clone());
    }
    registry
}

#[allow(dead_code)]
static BUILTIN_FUNCTION_REGISTRY: std::sync::OnceLock<FunctionRegistry> =
    std::sync::OnceLock::new();

#[allow(dead_code)]
pub fn builtin_function_registry() -> &'static FunctionRegistry {
    BUILTIN_FUNCTION_REGISTRY.get_or_init(|| build_function_registry(&[]))
}

#[allow(dead_code)]
pub fn lookup_function(name: &str) -> Option<&'static FunctionSpec> {
    builtin_function_registry().get(name)
}
