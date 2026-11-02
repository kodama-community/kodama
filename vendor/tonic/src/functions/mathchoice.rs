use crate::ast::ParseNode;
use crate::error::ParseError;
use crate::function_registry::{FunctionContext, FunctionParser, FunctionSpec};

use super::{ord_argument, require_function_arg};

pub(crate) fn mathchoice_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec!["\\mathchoice".to_string()],
        num_args: 4,
        primitive: true,
        handler: Some(mathchoice_handler),
        ..Default::default()
    }
}

fn mathchoice_handler(
    _parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    Ok(ParseNode::MathChoice {
        mode: context.mode,
        display: ord_argument(require_function_arg(args, 0, &context.func_name)?),
        text: ord_argument(require_function_arg(args, 1, &context.func_name)?),
        script: ord_argument(require_function_arg(args, 2, &context.func_name)?),
        scriptscript: ord_argument(require_function_arg(args, 3, &context.func_name)?),
    })
}
