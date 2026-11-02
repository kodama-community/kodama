use crate::builtin_macros::MathSymbolGroup;
use crate::error::ParseError;
use crate::lexer::starts_with_at;
use crate::macro_expander::{MacroExpander, MacroReplacement};

fn dots_by_token(name: &str) -> Option<String> {
    let result = match name {
        "," => Some("\\dotsc"),
        "\\not" => Some("\\dotsb"),
        "+"
        | "="
        | "<"
        | ">"
        | "-"
        | "*"
        | ":"
        | "\\DOTSB"
        | "\\coprod"
        | "\\bigvee"
        | "\\bigwedge"
        | "\\biguplus"
        | "\\bigcap"
        | "\\bigcup"
        | "\\prod"
        | "\\sum"
        | "\\bigotimes"
        | "\\bigoplus"
        | "\\bigodot"
        | "\\bigsqcup"
        | "\\And"
        | "\\longrightarrow"
        | "\\Longrightarrow"
        | "\\longleftarrow"
        | "\\Longleftarrow"
        | "\\longleftrightarrow"
        | "\\Longleftrightarrow"
        | "\\mapsto"
        | "\\longmapsto"
        | "\\hookrightarrow"
        | "\\doteq"
        | "\\mathbin"
        | "\\mathrel"
        | "\\relbar"
        | "\\Relbar"
        | "\\xrightarrow"
        | "\\xleftarrow" => Some("\\dotsb"),
        "\\DOTSI" | "\\int" | "\\oint" | "\\iint" | "\\iiint" | "\\iiiint" | "\\idotsint" => {
            Some("\\dotsi")
        }
        "\\DOTSX" => Some("\\dotsx"),
        _ => None,
    };
    result.map(ToString::to_string)
}

fn space_after_dots(name: &str) -> bool {
    matches!(
        name,
        ")" | "]"
            | "\\rbrack"
            | "\\}"
            | "\\rbrace"
            | "\\rangle"
            | "\\rceil"
            | "\\rfloor"
            | "\\rgroup"
            | "\\rmoustache"
            | "\\right"
            | "\\bigr"
            | "\\biggr"
            | "\\Bigr"
            | "\\Biggr"
            | "$"
            | ";"
            | "."
            | ","
    )
}

pub(crate) fn dots_macro(context: &mut MacroExpander) -> Result<MacroReplacement, ParseError> {
    let next = context.expand_after_future()?.text;
    let result = if let Some(value) = dots_by_token(&next) {
        value
    } else if starts_with_at(&next.chars().collect::<Vec<char>>(), 0, "\\not") {
        "\\dotsb".to_string()
    } else {
        match (context.math_symbol_group)(&next) {
            Some(MathSymbolGroup::BinarySymbol | MathSymbolGroup::RelationSymbol) => {
                "\\dotsb".to_string()
            }
            None => "\\dotso".to_string(),
        }
    };
    Ok(MacroReplacement::ReplacementText(result))
}

pub(crate) fn dots_other_macro(
    context: &mut MacroExpander,
) -> Result<MacroReplacement, ParseError> {
    if space_after_dots(&context.future()?.text) {
        Ok(MacroReplacement::ReplacementText("\\ldots\\,".to_string()))
    } else {
        Ok(MacroReplacement::ReplacementText("\\ldots".to_string()))
    }
}

pub(crate) fn dots_comma_macro(
    context: &mut MacroExpander,
) -> Result<MacroReplacement, ParseError> {
    let next = context.future()?;
    if next.text != "," && space_after_dots(&next.text) {
        Ok(MacroReplacement::ReplacementText("\\ldots\\,".to_string()))
    } else {
        Ok(MacroReplacement::ReplacementText("\\ldots".to_string()))
    }
}

pub(crate) fn centered_dots_macro(
    context: &mut MacroExpander,
) -> Result<MacroReplacement, ParseError> {
    if space_after_dots(&context.future()?.text) {
        Ok(MacroReplacement::ReplacementText("\\@cdots\\,".to_string()))
    } else {
        Ok(MacroReplacement::ReplacementText("\\@cdots".to_string()))
    }
}
