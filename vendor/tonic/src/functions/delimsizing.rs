use crate::ast::{AtomFamily, ParseNode};
use crate::error::ParseError;
use crate::function_registry::{ArgType, FunctionContext, FunctionParser, FunctionSpec};

use super::require_function_arg;

fn delimiter_size(func_name: &str) -> Result<usize, ParseError> {
    match func_name {
        "\\bigl" | "\\bigr" | "\\bigm" | "\\big" => Ok(1),
        "\\Bigl" | "\\Bigr" | "\\Bigm" | "\\Big" => Ok(2),
        "\\biggl" | "\\biggr" | "\\biggm" | "\\bigg" => Ok(3),
        "\\Biggl" | "\\Biggr" | "\\Biggm" | "\\Bigg" => Ok(4),
        _ => Err(ParseError::InternalInvariant {
            message: format!("Unknown delimiter sizing function {func_name}"),
        }),
    }
}

fn delimiter_mclass(func_name: &str) -> Result<AtomFamily, ParseError> {
    match func_name {
        "\\bigl" | "\\Bigl" | "\\biggl" | "\\Biggl" => Ok(AtomFamily::Mopen),
        "\\bigr" | "\\Bigr" | "\\biggr" | "\\Biggr" => Ok(AtomFamily::Mclose),
        "\\bigm" | "\\Bigm" | "\\biggm" | "\\Biggm" => Ok(AtomFamily::Mrel),
        "\\big" | "\\Big" | "\\bigg" | "\\Bigg" => Ok(AtomFamily::Mord),
        _ => Err(ParseError::InternalInvariant {
            message: format!("Unknown delimiter sizing function {func_name}"),
        }),
    }
}

fn delimiter_text(node: &ParseNode) -> Option<String> {
    match node {
        ParseNode::Atom { text, .. }
        | ParseNode::MathOrd { text, .. }
        | ParseNode::TextOrd { text, .. }
        | ParseNode::Spacing { text, .. }
        | ParseNode::AccentToken { text, .. }
        | ParseNode::OperatorToken { text, .. } => Some(text.clone()),
        _ => None,
    }
}

fn is_delimiter(text: &str) -> bool {
    matches!(
        text,
        "(" | "\\lparen"
            | ")"
            | "\\rparen"
            | "["
            | "\\lbrack"
            | "]"
            | "\\rbrack"
            | "\\{"
            | "\\lbrace"
            | "\\}"
            | "\\rbrace"
            | "\\lfloor"
            | "\\rfloor"
            | "⌊"
            | "⌋"
            | "\\lceil"
            | "\\rceil"
            | "⌈"
            | "⌉"
            | "<"
            | ">"
            | "\\langle"
            | "⟨"
            | "\\rangle"
            | "⟩"
            | "\\lt"
            | "\\gt"
            | "\\lvert"
            | "\\rvert"
            | "\\lVert"
            | "\\rVert"
            | "\\lgroup"
            | "\\rgroup"
            | "⟮"
            | "⟯"
            | "\\lmoustache"
            | "\\rmoustache"
            | "⎰"
            | "⎱"
            | "/"
            | "\\backslash"
            | "|"
            | "\\vert"
            | "\\|"
            | "\\Vert"
            | "\\uparrow"
            | "\\Uparrow"
            | "\\downarrow"
            | "\\Downarrow"
            | "\\updownarrow"
            | "\\Updownarrow"
            | "."
    )
}

fn checked_delimiter(node: &ParseNode, func_name: &str) -> Result<String, ParseError> {
    match delimiter_text(node) {
        Some(text) if is_delimiter(&text) => Ok(text),
        Some(text) => Err(ParseError::InvalidArgument {
            message: format!("Invalid delimiter '{text}' after '{func_name}'"),
            loc: None,
        }),
        None => Err(ParseError::InvalidArgument {
            message: format!("Invalid delimiter type after '{func_name}'"),
            loc: None,
        }),
    }
}

pub(crate) fn delim_sizing_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec![
            "\\bigl".to_string(),
            "\\Bigl".to_string(),
            "\\biggl".to_string(),
            "\\Biggl".to_string(),
            "\\bigr".to_string(),
            "\\Bigr".to_string(),
            "\\biggr".to_string(),
            "\\Biggr".to_string(),
            "\\bigm".to_string(),
            "\\Bigm".to_string(),
            "\\biggm".to_string(),
            "\\Biggm".to_string(),
            "\\big".to_string(),
            "\\Big".to_string(),
            "\\bigg".to_string(),
            "\\Bigg".to_string(),
        ],
        num_args: 1,
        arg_types: vec![ArgType::PrimitiveArg],
        handler: Some(delim_sizing_handler),
        ..Default::default()
    }
}

fn delim_sizing_handler(
    _parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    Ok(ParseNode::DelimSizing {
        mode: context.mode,
        size: delimiter_size(&context.func_name)?,
        mclass: delimiter_mclass(&context.func_name)?,
        delim: checked_delimiter(
            &require_function_arg(args, 0, &context.func_name)?,
            &context.func_name,
        )?,
    })
}

pub(crate) fn left_right_closing_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec!["\\right".to_string()],
        num_args: 1,
        arg_types: vec![ArgType::PrimitiveArg],
        primitive: true,
        handler: Some(left_right_closing_handler),
        ..Default::default()
    }
}

fn left_right_closing_handler(
    parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    Ok(ParseNode::LeftRightRight {
        mode: context.mode,
        delim: checked_delimiter(
            &require_function_arg(args, 0, &context.func_name)?,
            &context.func_name,
        )?,
        color: parser.current_color()?,
    })
}

pub(crate) fn left_right_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec!["\\left".to_string()],
        num_args: 1,
        arg_types: vec![ArgType::PrimitiveArg],
        primitive: true,
        handler: Some(left_right_handler),
        ..Default::default()
    }
}

fn left_right_handler(
    parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    parser.parse_left_right(&checked_delimiter(
        &require_function_arg(args, 0, &context.func_name)?,
        &context.func_name,
    )?)
}

pub(crate) fn middle_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec!["\\middle".to_string()],
        num_args: 1,
        arg_types: vec![ArgType::PrimitiveArg],
        primitive: true,
        handler: Some(middle_handler),
        ..Default::default()
    }
}

fn middle_handler(
    parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    let delim = checked_delimiter(
        &require_function_arg(args, 0, &context.func_name)?,
        &context.func_name,
    )?;
    if !parser.in_left_right() {
        return Err(ParseError::InvalidArgument {
            message: "\\middle without preceding \\left".to_string(),
            loc: None,
        });
    }
    Ok(ParseNode::Middle {
        mode: context.mode,
        delim,
    })
}
