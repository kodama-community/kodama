use crate::ast::ParseNode;
use crate::error::ParseError;
use crate::function_registry::{FunctionContext, FunctionParser, FunctionSpec};

use super::{binrel_class, is_character_box, normalize_argument, require_function_arg};

pub(crate) const FONT_COMMANDS: &[(&str, &str)] = &[
    ("\\mathrm", "mathrm"),
    ("\\mathit", "mathit"),
    ("\\mathbf", "mathbf"),
    ("\\mathnormal", "mathnormal"),
    ("\\mathsfit", "mathsfit"),
    ("\\mathbb", "mathbb"),
    ("\\mathcal", "mathcal"),
    ("\\mathfrak", "mathfrak"),
    ("\\mathscr", "mathscr"),
    ("\\mathsf", "mathsf"),
    ("\\mathtt", "mathtt"),
    ("\\Bbb", "mathbb"),
    ("\\bold", "mathbf"),
    ("\\frak", "mathfrak"),
];

pub(crate) const OLD_FONT_COMMANDS: &[(&str, &str)] = &[
    ("\\rm", "mathrm"),
    ("\\sf", "mathsf"),
    ("\\tt", "mathtt"),
    ("\\bf", "mathbf"),
    ("\\it", "mathit"),
    ("\\cal", "mathcal"),
];

fn command_font(commands: &[(&str, &str)], func_name: &str) -> Result<String, ParseError> {
    for (name, font) in commands {
        if *name == func_name {
            return Ok((*font).to_string());
        }
    }
    Err(ParseError::InternalInvariant {
        message: format!("Unknown font command: {func_name}"),
    })
}

pub(crate) fn font_spec() -> FunctionSpec {
    FunctionSpec {
        names: FONT_COMMANDS.iter().map(|(n, _)| n.to_string()).collect(),
        num_args: 1,
        allowed_in_argument: true,
        handler: Some(font_handler),
        ..Default::default()
    }
}

fn font_handler(
    _parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    Ok(ParseNode::Font {
        mode: context.mode,
        font: command_font(FONT_COMMANDS, &context.func_name)?,
        body: Box::new(normalize_argument(require_function_arg(
            args,
            0,
            &context.func_name,
        )?)),
    })
}

pub(crate) fn boldsymbol_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec!["\\boldsymbol".to_string(), "\\bm".to_string()],
        num_args: 1,
        handler: Some(boldsymbol_handler),
        ..Default::default()
    }
}

fn boldsymbol_handler(
    _parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    let body = require_function_arg(args, 0, &context.func_name)?;
    Ok(ParseNode::MClass {
        mode: context.mode,
        mclass: binrel_class(&body),
        body: vec![ParseNode::Font {
            mode: context.mode,
            font: "boldsymbol".to_string(),
            body: Box::new(body.clone()),
        }],
        is_character_box: is_character_box(&body),
    })
}

pub(crate) fn old_font_spec() -> FunctionSpec {
    FunctionSpec {
        names: OLD_FONT_COMMANDS
            .iter()
            .map(|(n, _)| n.to_string())
            .collect(),
        allowed_in_text: true,
        handler: Some(old_font_handler),
        ..Default::default()
    }
}

fn old_font_handler(
    parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    _args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    let body = parser.parse_expression(true, context.break_on_token_text.as_deref())?;
    Ok(ParseNode::Font {
        mode: context.mode,
        font: command_font(OLD_FONT_COMMANDS, &context.func_name)?,
        body: Box::new(ParseNode::OrdGroup {
            mode: context.mode,
            loc: None,
            body,
            semisimple: false,
        }),
    })
}
