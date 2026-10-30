use crate::ast::ParseNode;
use crate::error::ParseError;
use crate::function_registry::{FunctionContext, FunctionParser, FunctionSpec};

use super::require_function_arg;

pub(crate) fn x_arrow_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec![
            "\\xleftarrow".to_string(),
            "\\xrightarrow".to_string(),
            "\\xLeftarrow".to_string(),
            "\\xRightarrow".to_string(),
            "\\xleftrightarrow".to_string(),
            "\\xLeftrightarrow".to_string(),
            "\\xhookleftarrow".to_string(),
            "\\xhookrightarrow".to_string(),
            "\\xmapsto".to_string(),
            "\\xrightharpoondown".to_string(),
            "\\xrightharpoonup".to_string(),
            "\\xleftharpoondown".to_string(),
            "\\xleftharpoonup".to_string(),
            "\\xrightleftharpoons".to_string(),
            "\\xleftrightharpoons".to_string(),
            "\\xlongequal".to_string(),
            "\\xtwoheadrightarrow".to_string(),
            "\\xtwoheadleftarrow".to_string(),
            "\\xtofrom".to_string(),
            "\\xrightleftarrows".to_string(),
            "\\xrightequilibrium".to_string(),
            "\\xleftequilibrium".to_string(),
            "\\\\cdrightarrow".to_string(),
            "\\\\cdleftarrow".to_string(),
            "\\\\cdlongequal".to_string(),
        ],
        num_args: 1,
        num_optional_args: 1,
        handler: Some(x_arrow_handler),
        ..Default::default()
    }
}

fn x_arrow_handler(
    _parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    Ok(ParseNode::XArrow {
        mode: context.mode,
        label: context.func_name.clone(),
        body: Box::new(require_function_arg(args, 0, &context.func_name)?),
        below: opt_args
            .first()
            .and_then(std::clone::Clone::clone)
            .map(Box::new),
    })
}
