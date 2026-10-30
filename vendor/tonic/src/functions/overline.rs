use crate::ast::ParseNode;
use crate::error::ParseError;
use crate::function_registry::{FunctionContext, FunctionParser, FunctionSpec};

use super::require_function_arg;

pub(crate) fn overline_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec!["\\overline".to_string()],
        num_args: 1,
        handler: Some(overline_handler),
        ..Default::default()
    }
}

fn overline_handler(
    _parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    Ok(ParseNode::Overline {
        mode: context.mode,
        body: Box::new(require_function_arg(args, 0, &context.func_name)?),
    })
}
