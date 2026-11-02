use crate::ast::ParseNode;
use crate::error::ParseError;
use crate::function_registry::{FunctionContext, FunctionParser, FunctionSpec};

use super::require_function_arg;

pub(crate) fn horiz_brace_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec![
            "\\overbrace".to_string(),
            "\\underbrace".to_string(),
            "\\overbracket".to_string(),
            "\\underbracket".to_string(),
        ],
        num_args: 1,
        handler: Some(horiz_brace_handler),
        ..Default::default()
    }
}

fn is_over_brace(func_name: &str) -> Result<bool, ParseError> {
    match func_name {
        "\\overbrace" | "\\overbracket" => Ok(true),
        "\\underbrace" | "\\underbracket" => Ok(false),
        _ => Err(ParseError::InternalInvariant {
            message: format!("Unknown horizontal brace command: {func_name}"),
        }),
    }
}

fn horiz_brace_handler(
    _parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    Ok(ParseNode::HorizBrace {
        mode: context.mode,
        label: context.func_name.clone(),
        is_over: is_over_brace(&context.func_name)?,
        base: Box::new(require_function_arg(args, 0, &context.func_name)?),
    })
}
