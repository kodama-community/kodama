use crate::ast::ParseNode;
use crate::error::ParseError;
use crate::function_registry::{FunctionContext, FunctionParser, FunctionSpec};

pub(crate) fn sqrt_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec!["\\sqrt".to_string()],
        num_args: 1,
        num_optional_args: 1,
        primitive_after_missing_optional: Some(0),
        handler: Some(sqrt_handler),
        ..Default::default()
    }
}

fn sqrt_handler(
    _parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    let Some(body) = args.first() else {
        return Err(ParseError::InternalInvariant {
            message: "Missing body argument for \\sqrt".to_string(),
        });
    };
    let Some(index) = opt_args.first() else {
        return Err(ParseError::InternalInvariant {
            message: "Missing optional argument slot for \\sqrt".to_string(),
        });
    };
    Ok(ParseNode::Sqrt {
        mode: context.mode,
        body: Box::new(body.clone()),
        index: index.clone().map(Box::new),
    })
}
