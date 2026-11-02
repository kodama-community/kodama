use crate::ast::ParseNode;
use crate::error::ParseError;
use crate::function_registry::{FunctionContext, FunctionParser, FunctionSpec};

pub(crate) fn cr_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec!["\\\\".to_string()],
        allowed_in_text: true,
        handler: Some(cr_handler),
        ..Default::default()
    }
}

fn cr_handler(
    parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    _args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    let new_line = !context.display_mode
        || !parser.use_strict_behavior(
            "newLineInDisplayMode",
            "In LaTeX, \\\\ or \\newline does nothing in display mode",
            context.token.as_ref(),
        );
    Ok(ParseNode::Cr {
        mode: context.mode,
        new_line,
        size: parser.parse_optional_size()?,
    })
}
