use crate::ast::ParseNode;
use crate::error::ParseError;
use crate::function_registry::{FunctionContext, FunctionParser, FunctionSpec};

use super::{ord_argument, require_function_arg};

pub(crate) fn html_mathml_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec!["\\html@mathml".to_string()],
        num_args: 2,
        allowed_in_argument: true,
        allowed_in_text: true,
        handler: Some(html_mathml_handler),
        ..Default::default()
    }
}

fn html_mathml_handler(
    _parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    Ok(ParseNode::HtmlMathML {
        mode: context.mode,
        html: ord_argument(require_function_arg(args, 0, &context.func_name)?),
        mathml: ord_argument(require_function_arg(args, 1, &context.func_name)?),
    })
}
