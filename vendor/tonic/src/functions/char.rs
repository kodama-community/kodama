use crate::ast::ParseNode;
use crate::error::ParseError;
use crate::function_registry::{FunctionContext, FunctionParser, FunctionSpec};

use super::require_function_arg;

fn decimal_char_code(text: &str) -> Option<u32> {
    let mut value: u32 = 0;
    let mut saw_digit = false;
    for c in text.chars() {
        if !c.is_ascii_digit() {
            break;
        }
        saw_digit = true;
        let digit = c as u32 - '0' as u32;
        value = if value > (2147483647 - digit) / 10 {
            2147483647
        } else {
            value * 10 + digit
        };
    }
    if saw_digit { Some(value) } else { None }
}

fn char_argument_text(arg: &ParseNode, func_name: &str) -> Result<String, ParseError> {
    let body = match arg {
        ParseNode::OrdGroup { body, .. } => body,
        _ => {
            return Err(ParseError::InternalInvariant {
                message: format!("Expected group argument for {func_name}"),
            });
        }
    };
    let mut builder = String::new();
    for node in body {
        match node {
            ParseNode::TextOrd { text, .. } => builder.push_str(text),
            _ => {
                return Err(ParseError::InternalInvariant {
                    message: format!("Expected text character for {func_name}"),
                });
            }
        }
    }
    Ok(builder)
}

pub(crate) fn char_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec!["\\@char".to_string()],
        num_args: 1,
        allowed_in_text: true,
        handler: Some(char_handler),
        ..Default::default()
    }
}

fn char_handler(
    _parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    let number = char_argument_text(
        &require_function_arg(args, 0, &context.func_name)?,
        &context.func_name,
    )?;
    let Some(code) = decimal_char_code(&number) else {
        return Err(ParseError::InvalidArgument {
            message: format!("\\@char has non-numeric argument {number}"),
            loc: None,
        });
    };
    if code >= 0x10ffff {
        return Err(ParseError::InvalidArgument {
            message: format!("\\@char with invalid code point {number}"),
            loc: None,
        });
    }
    let Some(character) = char::from_u32(code) else {
        return Err(ParseError::InternalInvariant {
            message: "Unable to convert validated character code".to_string(),
        });
    };
    Ok(ParseNode::TextOrd {
        mode: context.mode,
        loc: None,
        text: character.to_string(),
    })
}
