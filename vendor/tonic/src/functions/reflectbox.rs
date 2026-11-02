use crate::ast::ParseNode;
use crate::error::ParseError;
use crate::function_registry::{ArgType, FunctionContext, FunctionParser, FunctionSpec};

use super::require_function_arg;

pub(crate) fn reflectbox_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec!["\\reflectbox".to_string()],
        num_args: 1,
        arg_types: vec![ArgType::HboxArg],
        allowed_in_text: true,
        handler: Some(reflectbox_handler),
        ..Default::default()
    }
}

pub(crate) fn mathreflectbox_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec!["\\mathreflectbox".to_string()],
        num_args: 1,
        arg_types: vec![ArgType::MathArg],
        handler: Some(reflectbox_handler),
        ..Default::default()
    }
}

fn reflectbox_handler(
    _parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    Ok(ParseNode::ReflectBox {
        mode: context.mode,
        body: Box::new(require_function_arg(args, 0, &context.func_name)?),
    })
}
