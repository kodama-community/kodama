use crate::ast::{LapAlignment, ParseNode};
use crate::error::ParseError;
use crate::function_registry::{FunctionContext, FunctionParser, FunctionSpec};

use super::require_function_arg;

fn lap_alignment(func_name: &str) -> Result<LapAlignment, ParseError> {
    match func_name {
        "\\mathllap" => Ok(LapAlignment::LLap),
        "\\mathrlap" => Ok(LapAlignment::RLap),
        "\\mathclap" => Ok(LapAlignment::CLap),
        _ => Err(ParseError::InternalInvariant {
            message: format!("Unknown lap command: {func_name}"),
        }),
    }
}

pub(crate) fn lap_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec![
            "\\mathllap".to_string(),
            "\\mathrlap".to_string(),
            "\\mathclap".to_string(),
        ],
        num_args: 1,
        allowed_in_text: true,
        handler: Some(lap_handler),
        ..Default::default()
    }
}

fn lap_handler(
    _parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    Ok(ParseNode::Lap {
        mode: context.mode,
        alignment: lap_alignment(&context.func_name)?,
        body: Box::new(require_function_arg(args, 0, &context.func_name)?),
    })
}
