use crate::ast::{Measurement, ParseNode};
use crate::error::ParseError;
use crate::function_registry::{ArgType, FunctionContext, FunctionParser, FunctionSpec};

use super::require_function_arg;

pub(crate) fn require_size_argument(
    arg: ParseNode,
    func_name: &str,
) -> Result<Measurement, ParseError> {
    match arg {
        ParseNode::Size { value, .. } => Ok(value),
        _ => Err(ParseError::InternalInvariant {
            message: format!("Expected size argument for {func_name}"),
        }),
    }
}

pub(crate) fn rule_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec!["\\rule".to_string()],
        num_args: 2,
        num_optional_args: 1,
        arg_types: vec![ArgType::SizeArg, ArgType::SizeArg, ArgType::SizeArg],
        allowed_in_text: true,
        handler: Some(rule_handler),
        ..Default::default()
    }
}

fn rule_handler(
    _parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    let shift = opt_args
        .first()
        .and_then(|o| o.as_ref())
        .map(|arg| require_size_argument(arg.clone(), &context.func_name))
        .transpose()?;
    Ok(ParseNode::Rule {
        mode: context.mode,
        shift,
        width: require_size_argument(
            require_function_arg(args, 0, &context.func_name)?,
            &context.func_name,
        )?,
        height: require_size_argument(
            require_function_arg(args, 1, &context.func_name)?,
            &context.func_name,
        )?,
    })
}
