use crate::error::ParseError;
use crate::macro_definition::{MacroDefinition, MacroExpansion};
use crate::macro_expander::{MacroExpander, MacroReplacement};
use crate::token::Token;

fn digit_value(text: &str) -> Option<usize> {
    if text.len() != 1 {
        return None;
    }
    let c = text.chars().next().unwrap();
    if c.is_ascii_digit() {
        Some((c as u32 - '0' as u32) as usize)
    } else if ('a'..='f').contains(&c) {
        Some((c as u32 - 'a' as u32 + 10) as usize)
    } else if ('A'..='F').contains(&c) {
        Some((c as u32 - 'A' as u32 + 10) as usize)
    } else {
        None
    }
}

fn char_code_from_token(token: &Token) -> Result<i64, ParseError> {
    if token.text == "EOF" {
        Err(ParseError::InvalidArgument {
            message: "\\char` missing argument".to_string(),
            loc: token.loc.clone(),
        })
    } else if token.text.len() > 1 && token.text.starts_with('\\') {
        Ok(token.text.chars().nth(1).unwrap() as i64)
    } else {
        Ok(token.text.chars().next().unwrap() as i64)
    }
}

fn parse_char_number(
    context: &mut MacroExpander,
    first: &Token,
    base: usize,
) -> Result<i64, ParseError> {
    let Some(value) = digit_value(&first.text).filter(|v| *v < base) else {
        return Err(ParseError::InvalidArgument {
            message: format!("Invalid base-{base} digit {}", first.text),
            loc: first.loc.clone(),
        });
    };
    let mut number = value as i64;
    loop {
        let token = context.future()?;
        match digit_value(&token.text) {
            Some(digit) if digit < base => {
                let _ = context.pop_token()?;
                number = if number > (2147483647 - digit as i64) / base as i64 {
                    2147483647
                } else {
                    number * base as i64 + digit as i64
                };
                continue;
            }
            _ => break,
        }
    }
    Ok(number)
}

pub(crate) fn char_macro(context: &mut MacroExpander) -> Result<MacroReplacement, ParseError> {
    let first = context.pop_token()?;
    let number = if first.text == "`" {
        char_code_from_token(&context.pop_token()?)?
    } else {
        let (base, digit) = if first.text == "'" {
            (8, context.pop_token()?)
        } else if first.text == "\"" {
            (16, context.pop_token()?)
        } else {
            (10, first)
        };
        parse_char_number(context, &digit, base)?
    };
    Ok(MacroReplacement::ReplacementText(format!(
        "\\@char{{{number}}}"
    )))
}

pub(crate) fn parse_argument_count(text: &str) -> Option<usize> {
    let mut number: u64 = 0;
    let mut saw_digit = false;
    let mut trailing_space = false;
    for c in text.chars() {
        if c.is_ascii_digit() && !trailing_space {
            saw_digit = true;
            let digit = c as u32 - '0' as u32;
            number = if number > (2147483647 - digit as u64) / 10 {
                2147483647
            } else {
                number * 10 + digit as u64
            };
        } else if matches!(c, ' ' | '\t' | '\n' | '\r') {
            if saw_digit {
                trailing_space = true;
            }
        } else {
            return None;
        }
    }
    if saw_digit {
        Some(number as usize)
    } else {
        None
    }
}

fn define_command_macro(
    context: &mut MacroExpander,
    exists_ok: bool,
    nonexists_ok: bool,
    skip_if_exists: bool,
) -> Result<MacroReplacement, ParseError> {
    let name_arg = context.consume_arg(None)?.tokens;
    if name_arg.len() != 1 {
        return Err(ParseError::InvalidArgument {
            message: "\\newcommand's first argument must be a macro name".to_string(),
            loc: None,
        });
    }
    let name = name_arg[0].text.clone();
    let exists = context.is_defined(&name);
    if exists && !exists_ok {
        return Err(ParseError::InvalidArgument {
            message: format!(
                "\\newcommand{{{name}}} attempting to redefine {name}; use \\renewcommand"
            ),
            loc: None,
        });
    }
    if !exists && !nonexists_ok {
        return Err(ParseError::InvalidArgument {
            message: format!(
                "\\renewcommand{{{name}}} when command {name} does not yet exist; use \\newcommand"
            ),
            loc: None,
        });
    }
    let mut body = context.consume_arg(None)?.tokens;
    let mut num_args = 0;
    if body.len() == 1 && body[0].text == "[" {
        num_args = parse_optional_arg_count(context)?;
        body = context.consume_arg(None)?.tokens;
    }
    if !(exists && skip_if_exists) {
        context.macros.set(
            name,
            Some(MacroDefinition::expansion(MacroExpansion {
                tokens: body,
                num_args,
                delimiters: None,
                unexpandable: false,
            })),
            false,
        );
    }
    Ok(MacroReplacement::ReplacementText(String::new()))
}

fn parse_optional_arg_count(context: &mut MacroExpander) -> Result<usize, ParseError> {
    let mut builder = String::new();
    loop {
        let token = context.expand_next_token()?;
        if token.text == "]" || token.text == "EOF" {
            break;
        }
        builder.push_str(&token.text);
    }
    let count_text = builder;
    let Some(value) = parse_argument_count(&count_text) else {
        return Err(ParseError::InvalidArgument {
            message: format!("Invalid number of arguments: {count_text}"),
            loc: None,
        });
    };
    Ok(value)
}

pub(crate) fn new_command_macro(
    context: &mut MacroExpander,
) -> Result<MacroReplacement, ParseError> {
    define_command_macro(context, false, true, false)
}

pub(crate) fn renew_command_macro(
    context: &mut MacroExpander,
) -> Result<MacroReplacement, ParseError> {
    define_command_macro(context, true, false, false)
}

pub(crate) fn provide_command_macro(
    context: &mut MacroExpander,
) -> Result<MacroReplacement, ParseError> {
    define_command_macro(context, true, true, true)
}
