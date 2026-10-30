use crate::ast::ParseNode;
use crate::error::ParseError;
use crate::function_registry::{FunctionContext, FunctionParser, FunctionSpec};

use super::{ord_argument, require_function_arg};

pub(crate) fn phantom_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec!["\\phantom".to_string()],
        num_args: 1,
        allowed_in_text: true,
        handler: Some(phantom_handler),
        ..Default::default()
    }
}

fn phantom_handler(
    _parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    Ok(ParseNode::Phantom {
        mode: context.mode,
        body: ord_argument(require_function_arg(args, 0, &context.func_name)?),
    })
}

pub(crate) fn vphantom_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec!["\\vphantom".to_string()],
        num_args: 1,
        allowed_in_text: true,
        handler: Some(vphantom_handler),
        ..Default::default()
    }
}

fn vphantom_handler(
    _parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    Ok(ParseNode::VPhantom {
        mode: context.mode,
        body: Box::new(require_function_arg(args, 0, &context.func_name)?),
    })
}
