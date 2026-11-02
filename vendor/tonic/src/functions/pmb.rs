use crate::ast::ParseNode;
use crate::error::ParseError;
use crate::function_registry::{FunctionContext, FunctionParser, FunctionSpec};

use super::{binrel_class, ord_argument, require_function_arg};

pub(crate) fn pmb_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec!["\\pmb".to_string()],
        num_args: 1,
        allowed_in_text: true,
        handler: Some(pmb_handler),
        ..Default::default()
    }
}

fn pmb_handler(
    _parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    let arg = require_function_arg(args, 0, &context.func_name)?;
    Ok(ParseNode::Pmb {
        mode: context.mode,
        mclass: binrel_class(&arg),
        body: ord_argument(arg),
    })
}
