use std::rc::Rc;

use crate::error::ParseError;
use crate::macro_expander::{MacroExpander, MacroHandler, MacroReplacement, token_expansion};
use crate::token::Token;

use crate::builtin_macros::MacroLogEvent;

fn tokens_to_text(tokens: &[Token]) -> String {
    tokens
        .iter()
        .rev()
        .map(|token| token.text.clone())
        .collect()
}

pub(crate) fn message_macro(context: &mut MacroExpander) -> Result<MacroReplacement, ParseError> {
    let args = context.consume_args(1, None)?;
    (context.macro_reporter)(MacroLogEvent::MacroMessage(tokens_to_text(&args[0])));
    Ok(MacroReplacement::ReplacementText(String::new()))
}

pub(crate) fn error_message_macro(
    context: &mut MacroExpander,
) -> Result<MacroReplacement, ParseError> {
    let args = context.consume_args(1, None)?;
    (context.macro_reporter)(MacroLogEvent::MacroErrorMessage(tokens_to_text(&args[0])));
    Ok(MacroReplacement::ReplacementText(String::new()))
}

pub(crate) fn show_macro(context: &mut MacroExpander) -> Result<MacroReplacement, ParseError> {
    let token = context.pop_token()?;
    let status = if context.is_defined(&token.text) {
        "defined"
    } else {
        "undefined"
    };
    (context.macro_reporter)(MacroLogEvent::MacroShow(format!(
        "{}: {}",
        token.text, status
    )));
    Ok(MacroReplacement::ReplacementText(String::new()))
}

pub(crate) fn tag_literal_macro(
    context: &mut MacroExpander,
) -> Result<MacroReplacement, ParseError> {
    if context.macros.has("\\df@tag") {
        return Err(ParseError::InvalidArgument {
            message: "Multiple \\tag".to_string(),
            loc: None,
        });
    }
    Ok(MacroReplacement::ReplacementText(
        "\\gdef\\df@tag{\\text{#1}}".to_string(),
    ))
}

fn restore_dynamic_macro(context: &mut MacroExpander, name: &str, old_value: Option<MacroHandler>) {
    context
        .dynamic_macros
        .set(name.to_string(), old_value, false);
}

fn braket_separator_handler(
    one: bool,
    doubled: bool,
    middle: Vec<Token>,
    middle_double: Vec<Token>,
    old_middle: Option<MacroHandler>,
    old_middle_double: Option<MacroHandler>,
) -> MacroHandler {
    Rc::new(move |context| {
        if one {
            restore_dynamic_macro(context, "|", old_middle.clone());
            if !middle_double.is_empty() {
                restore_dynamic_macro(context, "\\|", old_middle_double.clone());
            }
        }
        let mut use_double = doubled;
        if !doubled && !middle_double.is_empty() {
            let next = context.future()?;
            if next.text == "|" {
                let _ = context.pop_token()?;
                use_double = true;
            }
        }
        if use_double {
            Ok(token_expansion(middle_double.clone()))
        } else {
            Ok(token_expansion(middle.clone()))
        }
    })
}

fn braket_helper(context: &mut MacroExpander, one: bool) -> Result<MacroReplacement, ParseError> {
    let parts = context.consume_args(4, None)?;
    let left = parts[0].clone();
    let middle = parts[1].clone();
    let middle_double = parts[2].clone();
    let right = parts[3].clone();
    let old_middle = context.dynamic_macros.get_current("|").cloned();
    let old_middle_double = context.dynamic_macros.get_current("\\|").cloned();
    context.begin_group();
    context.dynamic_macros.set(
        "|".to_string(),
        Some(braket_separator_handler(
            one,
            false,
            middle.clone(),
            middle_double.clone(),
            old_middle.clone(),
            old_middle_double.clone(),
        )),
        false,
    );
    if !middle_double.is_empty() {
        context.dynamic_macros.set(
            "\\|".to_string(),
            Some(braket_separator_handler(
                one,
                true,
                middle,
                middle_double,
                old_middle,
                old_middle_double,
            )),
            false,
        );
    }
    let expanded: Result<Vec<Token>, ParseError> = (|| {
        let argument = context.consume_arg(None)?.tokens;
        let mut input = right.clone();
        input.extend(argument);
        input.extend(left.clone());
        context.expand_tokens(input)
    })();
    let close_result = context.end_group();
    match (expanded, close_result) {
        (Err(err), _) => Err(err),
        (Ok(_), Err(err)) => Err(err),
        (Ok(mut tokens), Ok(())) => {
            tokens.reverse();
            Ok(token_expansion(tokens))
        }
    }
}

pub(crate) fn braket_macro(context: &mut MacroExpander) -> Result<MacroReplacement, ParseError> {
    braket_helper(context, false)
}

pub(crate) fn set_macro(context: &mut MacroExpander) -> Result<MacroReplacement, ParseError> {
    braket_helper(context, true)
}
