use crate::ast::{ParseNode, StyleLevel};
use crate::error::ParseError;
use crate::function_registry::{FunctionContext, FunctionParser, FunctionSpec};
use crate::token::token_location;

pub(crate) fn math_mode_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec!["\\(".to_string(), "$".to_string()],
        allowed_in_text: true,
        allowed_in_math: false,
        handler: Some(math_mode_handler),
        ..Default::default()
    }
}

fn math_mode_handler(
    parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    _args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    let close = if context.func_name == "\\(" {
        "\\)"
    } else {
        "$"
    };
    Ok(ParseNode::Styling {
        mode: context.mode,
        style: StyleLevel::TextStyle,
        reset_font: true,
        body: parser.parse_math_mode(close)?,
    })
}

pub(crate) fn math_closing_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec!["\\)".to_string(), "\\]".to_string()],
        allowed_in_text: true,
        allowed_in_math: false,
        handler: Some(math_closing_handler),
        ..Default::default()
    }
}

fn math_closing_handler(
    _parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    _args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    let loc = token_location(context.token.as_ref());
    Err(ParseError::InvalidArgument {
        message: format!("Mismatched {}", context.func_name),
        loc,
    })
}
