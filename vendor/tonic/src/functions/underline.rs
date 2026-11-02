use crate::ast::ParseNode;
use crate::error::ParseError;
use crate::function_registry::{FunctionContext, FunctionParser, FunctionSpec};

use super::require_function_arg;

pub(crate) fn underline_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec!["\\underline".to_string()],
        num_args: 1,
        allowed_in_text: true,
        handler: Some(underline_handler),
        ..Default::default()
    }
}

fn underline_handler(
    _parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    Ok(ParseNode::Underline {
        mode: context.mode,
        body: Box::new(require_function_arg(args, 0, &context.func_name)?),
    })
}
