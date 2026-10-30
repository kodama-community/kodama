use crate::ast::ParseNode;
use crate::error::ParseError;
use crate::function_registry::{ArgType, FunctionContext, FunctionParser, FunctionSpec};

use super::{require_function_arg, require_size_argument};

pub(crate) fn raisebox_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec!["\\raisebox".to_string()],
        num_args: 2,
        arg_types: vec![ArgType::SizeArg, ArgType::HboxArg],
        allowed_in_text: true,
        handler: Some(raisebox_handler),
        ..Default::default()
    }
}

fn raisebox_handler(
    _parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    let amount = require_size_argument(
        require_function_arg(args, 0, &context.func_name)?,
        &context.func_name,
    )?;
    Ok(ParseNode::RaiseBox {
        mode: context.mode,
        dy: amount,
        body: Box::new(require_function_arg(args, 1, &context.func_name)?),
    })
}
