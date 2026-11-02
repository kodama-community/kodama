use crate::ast::ParseNode;
use crate::error::ParseError;
use crate::function_registry::{ArgType, FunctionContext, FunctionParser, FunctionSpec};

use super::require_function_arg;

pub(crate) fn vcenter_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec!["\\vcenter".to_string()],
        num_args: 1,
        arg_types: vec![ArgType::OriginalArg],
        allowed_in_math: true,
        allowed_in_text: false,
        handler: Some(vcenter_handler),
        ..Default::default()
    }
}

fn vcenter_handler(
    _parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    Ok(ParseNode::VCenter {
        mode: context.mode,
        body: Box::new(require_function_arg(args, 0, &context.func_name)?),
    })
}
