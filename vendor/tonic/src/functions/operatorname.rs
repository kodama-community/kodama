use crate::ast::ParseNode;
use crate::error::ParseError;
use crate::function_registry::{FunctionContext, FunctionParser, FunctionSpec};

use super::{ord_argument, require_function_arg};

pub(crate) fn operatorname_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec![
            "\\operatorname@".to_string(),
            "\\operatornamewithlimits".to_string(),
        ],
        num_args: 1,
        handler: Some(operatorname_handler),
        ..Default::default()
    }
}

fn operatorname_handler(
    _parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    Ok(ParseNode::OperatorName {
        mode: context.mode,
        body: ord_argument(require_function_arg(args, 0, &context.func_name)?),
        always_handle_sup_sub: context.func_name == "\\operatornamewithlimits",
        limits: false,
        parent_is_sup_sub: false,
    })
}
