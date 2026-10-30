use crate::ast::ParseNode;
use crate::error::ParseError;
use crate::function_registry::{ArgType, FunctionContext, FunctionParser, FunctionSpec};

use super::require_function_arg;

fn environment_name(arg: &ParseNode) -> Result<String, ParseError> {
    let body = match arg {
        ParseNode::OrdGroup { body, .. } => body,
        _ => {
            return Err(ParseError::InvalidArgument {
                message: "Invalid environment name".to_string(),
                loc: None,
            });
        }
    };
    let mut builder = String::new();
    for node in body {
        match node {
            ParseNode::TextOrd { text, .. } => builder.push_str(text),
            _ => {
                return Err(ParseError::InvalidArgument {
                    message: "Invalid environment name".to_string(),
                    loc: None,
                });
            }
        }
    }
    Ok(builder)
}

pub(crate) fn begin_end_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec!["\\begin".to_string(), "\\end".to_string()],
        num_args: 1,
        arg_types: vec![ArgType::TextArg],
        handler: Some(environment_handler),
        ..Default::default()
    }
}

fn environment_handler(
    parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    let name = environment_name(&require_function_arg(args, 0, &context.func_name)?)?;
    if context.func_name == "\\begin" {
        parser.parse_environment(&name)
    } else {
        Ok(ParseNode::EnvironmentEnd {
            mode: context.mode,
            name,
        })
    }
}

pub(crate) fn hline_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec!["\\hline".to_string(), "\\hdashline".to_string()],
        allowed_in_text: true,
        handler: Some(hline_handler),
        ..Default::default()
    }
}

fn hline_handler(
    _parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    _args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    Err(ParseError::InvalidArgument {
        message: format!("{} valid only within array environment", context.func_name),
        loc: None,
    })
}
