use crate::ast::ParseNode;
use crate::error::ParseError;
use crate::function_registry::{FunctionContext, FunctionParser, FunctionSpec};

pub(crate) const SIZING_COMMANDS: &[&str] = &[
    "\\tiny",
    "\\sixptsize",
    "\\scriptsize",
    "\\footnotesize",
    "\\small",
    "\\normalsize",
    "\\large",
    "\\Large",
    "\\LARGE",
    "\\huge",
    "\\Huge",
];

fn sizing_index(func_name: &str) -> Result<usize, ParseError> {
    for (index, command) in SIZING_COMMANDS.iter().enumerate() {
        if *command == func_name {
            return Ok(index + 1);
        }
    }
    Err(ParseError::InternalInvariant {
        message: format!("Unknown sizing command: {func_name}"),
    })
}

pub(crate) fn sizing_spec() -> FunctionSpec {
    FunctionSpec {
        names: SIZING_COMMANDS.iter().map(ToString::to_string).collect(),
        allowed_in_text: true,
        handler: Some(sizing_handler),
        ..Default::default()
    }
}

fn sizing_handler(
    parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    _args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    Ok(ParseNode::Sizing {
        mode: context.mode,
        size: sizing_index(&context.func_name)?,
        body: parser.parse_expression(false, context.break_on_token_text.as_deref())?,
    })
}
