use crate::ast::{AtomFamily, ParseNode, StyleLevel};
use crate::error::ParseError;
use crate::function_registry::{ArgType, FunctionContext, FunctionParser, FunctionSpec};
use crate::token::token_location;

use super::{normalize_argument, require_function_arg};

struct GenFracConfig {
    continued: bool,
    has_bar_line: bool,
    left_delim: Option<String>,
    right_delim: Option<String>,
    style: Option<StyleLevel>,
}

pub(crate) fn wrap_genfrac_style(node: ParseNode, style: Option<StyleLevel>) -> ParseNode {
    if let Some(style) = style {
        ParseNode::Styling {
            mode: node.mode(),
            body: vec![node],
            style,
            reset_font: false,
        }
    } else {
        node
    }
}

fn standard_genfrac_config(func_name: &str) -> Result<GenFracConfig, ParseError> {
    let config = match func_name {
        "\\cfrac" => GenFracConfig {
            continued: true,
            has_bar_line: true,
            left_delim: None,
            right_delim: None,
            style: Some(StyleLevel::DisplayStyle),
        },
        "\\dfrac" => GenFracConfig {
            continued: false,
            has_bar_line: true,
            left_delim: None,
            right_delim: None,
            style: Some(StyleLevel::DisplayStyle),
        },
        "\\frac" => GenFracConfig {
            continued: false,
            has_bar_line: true,
            left_delim: None,
            right_delim: None,
            style: None,
        },
        "\\tfrac" => GenFracConfig {
            continued: false,
            has_bar_line: true,
            left_delim: None,
            right_delim: None,
            style: Some(StyleLevel::TextStyle),
        },
        "\\dbinom" => GenFracConfig {
            continued: false,
            has_bar_line: false,
            left_delim: Some("(".to_string()),
            right_delim: Some(")".to_string()),
            style: Some(StyleLevel::DisplayStyle),
        },
        "\\binom" => GenFracConfig {
            continued: false,
            has_bar_line: false,
            left_delim: Some("(".to_string()),
            right_delim: Some(")".to_string()),
            style: None,
        },
        "\\tbinom" => GenFracConfig {
            continued: false,
            has_bar_line: false,
            left_delim: Some("(".to_string()),
            right_delim: Some(")".to_string()),
            style: Some(StyleLevel::TextStyle),
        },
        "\\\\atopfrac" => GenFracConfig {
            continued: false,
            has_bar_line: false,
            left_delim: None,
            right_delim: None,
            style: None,
        },
        "\\\\bracefrac" => GenFracConfig {
            continued: false,
            has_bar_line: false,
            left_delim: Some("\\{".to_string()),
            right_delim: Some("\\}".to_string()),
            style: None,
        },
        "\\\\brackfrac" => GenFracConfig {
            continued: false,
            has_bar_line: false,
            left_delim: Some("[".to_string()),
            right_delim: Some("]".to_string()),
            style: None,
        },
        _ => {
            return Err(ParseError::InternalInvariant {
                message: format!("Unrecognized standard genfrac command: {func_name}"),
            });
        }
    };
    Ok(config)
}

pub(crate) fn standard_genfrac_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec![
            "\\cfrac".to_string(),
            "\\dfrac".to_string(),
            "\\frac".to_string(),
            "\\tfrac".to_string(),
            "\\dbinom".to_string(),
            "\\binom".to_string(),
            "\\tbinom".to_string(),
            "\\\\atopfrac".to_string(),
            "\\\\bracefrac".to_string(),
            "\\\\brackfrac".to_string(),
        ],
        num_args: 2,
        allowed_in_argument: true,
        handler: Some(standard_genfrac_handler),
        ..Default::default()
    }
}

fn standard_genfrac_handler(
    _parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    let config = standard_genfrac_config(&context.func_name)?;
    let node = ParseNode::GenFrac {
        mode: context.mode,
        numer: Box::new(require_function_arg(args, 0, &context.func_name)?),
        denom: Box::new(require_function_arg(args, 1, &context.func_name)?),
        continued: config.continued,
        has_bar_line: config.has_bar_line,
        bar_size: None,
        left_delim: config.left_delim,
        right_delim: config.right_delim,
    };
    Ok(wrap_genfrac_style(node, config.style))
}

fn infix_replacement(func_name: &str) -> Result<String, ParseError> {
    let replacement = match func_name {
        "\\over" => "\\frac",
        "\\choose" => "\\binom",
        "\\atop" => "\\\\atopfrac",
        "\\brace" => "\\\\bracefrac",
        "\\brack" => "\\\\brackfrac",
        _ => {
            return Err(ParseError::InternalInvariant {
                message: format!("Unrecognized infix genfrac command: {func_name}"),
            });
        }
    };
    Ok(replacement.to_string())
}

pub(crate) fn infix_genfrac_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec![
            "\\over".to_string(),
            "\\choose".to_string(),
            "\\atop".to_string(),
            "\\brace".to_string(),
            "\\brack".to_string(),
        ],
        infix: true,
        handler: Some(infix_genfrac_handler),
        ..Default::default()
    }
}

fn infix_genfrac_handler(
    _parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    _args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    Ok(ParseNode::Infix {
        mode: context.mode,
        replace_with: infix_replacement(&context.func_name)?,
        size: None,
        loc: token_location(context.token.as_ref()),
    })
}

fn delimiter_from_argument(arg: ParseNode, family: AtomFamily) -> Option<String> {
    match normalize_argument(arg) {
        ParseNode::Atom {
            family: AtomFamily::Mopen,
            text,
            ..
        } if family == AtomFamily::Mopen => {
            if text == "." {
                None
            } else {
                Some(text)
            }
        }
        ParseNode::Atom {
            family: AtomFamily::Mclose,
            text,
            ..
        } if family == AtomFamily::Mclose => {
            if text == "." {
                None
            } else {
                Some(text)
            }
        }
        _ => None,
    }
}

fn genfrac_style(arg: &ParseNode) -> Result<Option<StyleLevel>, ParseError> {
    let first = match arg {
        ParseNode::OrdGroup { body, .. } if body.is_empty() => return Ok(None),
        ParseNode::OrdGroup { body, .. } => &body[0],
        node => node,
    };
    let text = match first {
        ParseNode::TextOrd { text, .. } => text,
        _ => {
            return Err(ParseError::InternalInvariant {
                message: "\\genfrac style argument did not contain a textord".to_string(),
            });
        }
    };
    Ok(match text.as_str() {
        "0" => Some(StyleLevel::DisplayStyle),
        "1" => Some(StyleLevel::TextStyle),
        "2" => Some(StyleLevel::ScriptStyle),
        "3" => Some(StyleLevel::ScriptScriptStyle),
        _ => None,
    })
}

pub(crate) fn general_genfrac_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec!["\\genfrac".to_string()],
        num_args: 6,
        arg_types: vec![
            ArgType::MathArg,
            ArgType::MathArg,
            ArgType::SizeArg,
            ArgType::TextArg,
            ArgType::MathArg,
            ArgType::MathArg,
        ],
        allowed_in_argument: true,
        handler: Some(general_genfrac_handler),
        ..Default::default()
    }
}

fn general_genfrac_handler(
    _parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    let left = require_function_arg(args, 0, &context.func_name)?;
    let right = require_function_arg(args, 1, &context.func_name)?;
    let bar = require_function_arg(args, 2, &context.func_name)?;
    let style_arg = require_function_arg(args, 3, &context.func_name)?;
    let numer = require_function_arg(args, 4, &context.func_name)?;
    let denom = require_function_arg(args, 5, &context.func_name)?;
    let (has_bar_line, bar_size) = match &bar {
        ParseNode::Size {
            value, is_blank, ..
        } => {
            let has_bar_line = *is_blank || value.number > 0.0;
            let bar_size = if *is_blank { None } else { Some(value.clone()) };
            (has_bar_line, bar_size)
        }
        _ => {
            return Err(ParseError::InternalInvariant {
                message: "\\genfrac bar argument was not a size".to_string(),
            });
        }
    };
    let node = ParseNode::GenFrac {
        mode: context.mode,
        numer: Box::new(numer),
        denom: Box::new(denom),
        continued: false,
        has_bar_line,
        bar_size,
        left_delim: delimiter_from_argument(left, AtomFamily::Mopen),
        right_delim: delimiter_from_argument(right, AtomFamily::Mclose),
    };
    Ok(wrap_genfrac_style(node, genfrac_style(&style_arg)?))
}

pub(crate) fn above_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec!["\\above".to_string()],
        num_args: 1,
        arg_types: vec![ArgType::SizeArg],
        infix: true,
        handler: Some(above_handler),
        ..Default::default()
    }
}

fn above_handler(
    _parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    let arg = require_function_arg(args, 0, &context.func_name)?;
    let value = match &arg {
        ParseNode::Size { value, .. } => value.clone(),
        _ => {
            return Err(ParseError::InternalInvariant {
                message: "\\above argument was not a size".to_string(),
            });
        }
    };
    Ok(ParseNode::Infix {
        mode: context.mode,
        replace_with: "\\\\abovefrac".to_string(),
        size: Some(value),
        loc: token_location(context.token.as_ref()),
    })
}

pub(crate) fn abovefrac_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec!["\\\\abovefrac".to_string()],
        num_args: 3,
        arg_types: vec![ArgType::MathArg, ArgType::SizeArg, ArgType::MathArg],
        handler: Some(abovefrac_handler),
        ..Default::default()
    }
}

fn abovefrac_handler(
    _parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    let numer = require_function_arg(args, 0, &context.func_name)?;
    let infix = require_function_arg(args, 1, &context.func_name)?;
    let denom = require_function_arg(args, 2, &context.func_name)?;
    let bar_size = match &infix {
        ParseNode::Infix {
            size: Some(size), ..
        } => size.clone(),
        _ => {
            return Err(ParseError::InternalInvariant {
                message: "\\\\abovefrac expected an infix size".to_string(),
            });
        }
    };
    Ok(ParseNode::GenFrac {
        mode: context.mode,
        numer: Box::new(numer),
        denom: Box::new(denom),
        continued: false,
        has_bar_line: bar_size.number > 0.0,
        bar_size: Some(bar_size),
        left_delim: None,
        right_delim: None,
    })
}
