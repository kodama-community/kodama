use crate::ast::Mode;
use crate::error::ParseError;
use crate::macro_expander::{MacroExpander, MacroReplacement, token_expansion};

pub(crate) fn noexpand_macro(context: &mut MacroExpander) -> Result<MacroReplacement, ParseError> {
    let mut token = context.pop_token()?;
    if context.is_expandable(&token.text) {
        token.noexpand = true;
        token.treat_as_relax = true;
    }
    Ok(token_expansion(vec![token]))
}

pub(crate) fn expandafter_macro(
    context: &mut MacroExpander,
) -> Result<MacroReplacement, ParseError> {
    let token = context.pop_token()?;
    let _ = context.expand_once(true)?;
    Ok(token_expansion(vec![token]))
}

pub(crate) fn first_of_two_macro(
    context: &mut MacroExpander,
) -> Result<MacroReplacement, ParseError> {
    let args = context.consume_args(2, None)?;
    let tokens = args
        .first()
        .cloned()
        .ok_or_else(|| ParseError::InternalInvariant {
            message: "Missing first macro argument".to_string(),
        })?;
    Ok(token_expansion(tokens))
}

pub(crate) fn second_of_two_macro(
    context: &mut MacroExpander,
) -> Result<MacroReplacement, ParseError> {
    let args = context.consume_args(2, None)?;
    let tokens = args
        .get(1)
        .cloned()
        .ok_or_else(|| ParseError::InternalInvariant {
            message: "Missing second macro argument".to_string(),
        })?;
    Ok(token_expansion(tokens))
}

pub(crate) fn if_next_char_macro(
    context: &mut MacroExpander,
) -> Result<MacroReplacement, ParseError> {
    let args = context.consume_args(3, None)?;
    context.consume_spaces()?;
    let next = context.future()?;
    let selected = if args[0].len() == 1 && args[0][0].text == next.text {
        args[1].clone()
    } else {
        args[2].clone()
    };
    Ok(token_expansion(selected))
}

pub(crate) fn text_or_math_macro(
    context: &mut MacroExpander,
) -> Result<MacroReplacement, ParseError> {
    let args = context.consume_args(2, None)?;
    if context.mode == Mode::Text {
        Ok(token_expansion(args[0].clone()))
    } else {
        Ok(token_expansion(args[1].clone()))
    }
}
