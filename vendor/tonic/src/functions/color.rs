use crate::ast::ParseNode;
use crate::error::ParseError;
use crate::function_registry::{ArgType, FunctionContext, FunctionParser, FunctionSpec};
use crate::macro_definition::MacroDefinition;

use super::{ord_argument, require_function_arg};

pub(crate) fn require_color_argument(
    args: &[ParseNode],
    func_name: &str,
) -> Result<String, ParseError> {
    require_color_argument_at(args, 0, func_name)
}

pub(crate) fn require_color_argument_at(
    args: &[ParseNode],
    index: usize,
    func_name: &str,
) -> Result<String, ParseError> {
    match require_function_arg(args, index, func_name)? {
        ParseNode::ColorToken { color, .. } => Ok(color),
        _ => Err(ParseError::InternalInvariant {
            message: format!("Expected color argument for {func_name}"),
        }),
    }
}

pub(crate) fn textcolor_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec!["\\textcolor".to_string()],
        num_args: 2,
        arg_types: vec![ArgType::ColorArg, ArgType::OriginalArg],
        allowed_in_text: true,
        handler: Some(textcolor_handler),
        ..Default::default()
    }
}

fn textcolor_handler(
    _parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    Ok(ParseNode::Color {
        mode: context.mode,
        color: require_color_argument(args, &context.func_name)?,
        body: ord_argument(require_function_arg(args, 1, &context.func_name)?),
    })
}

pub(crate) fn color_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec!["\\color".to_string()],
        num_args: 1,
        arg_types: vec![ArgType::ColorArg],
        allowed_in_text: true,
        handler: Some(color_handler),
        ..Default::default()
    }
}

fn color_handler(
    parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    let color = require_color_argument(args, &context.func_name)?;
    parser.set_macro(
        "\\current@color",
        Some(MacroDefinition::text(color.clone())),
    );
    Ok(ParseNode::Color {
        mode: context.mode,
        color,
        body: parser.parse_expression(true, context.break_on_token_text.as_deref())?,
    })
}
