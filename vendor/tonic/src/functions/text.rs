use crate::ast::ParseNode;
use crate::error::ParseError;
use crate::function_registry::{ArgType, FunctionContext, FunctionParser, FunctionSpec};

use super::{ord_argument, require_function_arg};

pub(crate) fn text_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec![
            "\\text".to_string(),
            "\\textrm".to_string(),
            "\\textsf".to_string(),
            "\\texttt".to_string(),
            "\\textnormal".to_string(),
            "\\textbf".to_string(),
            "\\textmd".to_string(),
            "\\textit".to_string(),
            "\\textup".to_string(),
            "\\emph".to_string(),
        ],
        num_args: 1,
        arg_types: vec![ArgType::TextArg],
        allowed_in_argument: true,
        allowed_in_text: true,
        handler: Some(text_handler),
        ..Default::default()
    }
}

fn text_handler(
    _parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    Ok(ParseNode::Text {
        mode: context.mode,
        body: ord_argument(require_function_arg(args, 0, &context.func_name)?),
        font: context.func_name.clone(),
    })
}
