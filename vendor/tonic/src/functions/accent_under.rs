use crate::ast::ParseNode;
use crate::error::ParseError;
use crate::function_registry::{FunctionContext, FunctionParser, FunctionSpec};

use super::require_function_arg;

pub(crate) fn accent_under_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec![
            "\\underleftarrow".to_string(),
            "\\underrightarrow".to_string(),
            "\\underleftrightarrow".to_string(),
            "\\undergroup".to_string(),
            "\\underlinesegment".to_string(),
            "\\utilde".to_string(),
        ],
        num_args: 1,
        handler: Some(accent_under_handler),
        ..Default::default()
    }
}

fn accent_under_handler(
    _parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    Ok(ParseNode::AccentUnder {
        mode: context.mode,
        label: context.func_name.clone(),
        base: Box::new(require_function_arg(args, 0, &context.func_name)?),
    })
}
