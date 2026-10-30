use crate::ast::{ParseNode, StyleLevel};
use crate::error::ParseError;
use crate::function_registry::{FunctionContext, FunctionParser, FunctionSpec};

fn style_level(func_name: &str) -> Result<StyleLevel, ParseError> {
    match func_name {
        "\\displaystyle" => Ok(StyleLevel::DisplayStyle),
        "\\textstyle" => Ok(StyleLevel::TextStyle),
        "\\scriptstyle" => Ok(StyleLevel::ScriptStyle),
        "\\scriptscriptstyle" => Ok(StyleLevel::ScriptScriptStyle),
        _ => Err(ParseError::InternalInvariant {
            message: format!("Unknown styling command: {func_name}"),
        }),
    }
}

pub(crate) fn styling_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec![
            "\\displaystyle".to_string(),
            "\\textstyle".to_string(),
            "\\scriptstyle".to_string(),
            "\\scriptscriptstyle".to_string(),
        ],
        allowed_in_text: true,
        primitive: true,
        handler: Some(styling_handler),
        ..Default::default()
    }
}

fn styling_handler(
    parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    _args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    Ok(ParseNode::Styling {
        mode: context.mode,
        body: parser.parse_expression(true, context.break_on_token_text.as_deref())?,
        style: style_level(&context.func_name)?,
        reset_font: false,
    })
}
