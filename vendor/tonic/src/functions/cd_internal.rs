use crate::ast::ParseNode;
use crate::error::ParseError;
use crate::function_registry::{FunctionContext, FunctionParser, FunctionSpec};

use super::require_function_arg;

pub(crate) fn cd_internal_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec!["\\\\cdleft".to_string(), "\\\\cdright".to_string()],
        num_args: 1,
        handler: Some(cd_label_handler),
        ..Default::default()
    }
}

fn cd_label_handler(
    _parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    Ok(ParseNode::CdLabel {
        mode: context.mode,
        side: if context.func_name == "\\\\cdleft" {
            "left".to_string()
        } else {
            "right".to_string()
        },
        label: Box::new(require_function_arg(args, 0, &context.func_name)?),
    })
}

pub(crate) fn cd_parent_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec!["\\\\cdparent".to_string()],
        num_args: 1,
        handler: Some(cd_parent_handler),
        ..Default::default()
    }
}

fn cd_parent_handler(
    _parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    Ok(ParseNode::CdParent {
        mode: context.mode,
        fragment: Box::new(require_function_arg(args, 0, &context.func_name)?),
    })
}
