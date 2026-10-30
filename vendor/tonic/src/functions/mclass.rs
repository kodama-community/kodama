use crate::ast::{AtomFamily, ParseNode};
use crate::error::ParseError;
use crate::function_registry::{FunctionContext, FunctionParser, FunctionSpec};

use super::{binrel_class, is_character_box, ord_argument, require_function_arg};

pub(crate) const MCLASS_COMMANDS: &[(&str, AtomFamily)] = &[
    ("\\mathord", AtomFamily::Mord),
    ("\\mathbin", AtomFamily::Mbin),
    ("\\mathrel", AtomFamily::Mrel),
    ("\\mathopen", AtomFamily::Mopen),
    ("\\mathclose", AtomFamily::Mclose),
    ("\\mathpunct", AtomFamily::Mpunct),
    ("\\mathinner", AtomFamily::Minner),
];

fn command_mclass(func_name: &str) -> Result<AtomFamily, ParseError> {
    for (name, mclass) in MCLASS_COMMANDS {
        if *name == func_name {
            return Ok(*mclass);
        }
    }
    Err(ParseError::InternalInvariant {
        message: format!("Unknown math class command: {func_name}"),
    })
}

pub(crate) fn mclass_spec() -> FunctionSpec {
    FunctionSpec {
        names: MCLASS_COMMANDS.iter().map(|(n, _)| n.to_string()).collect(),
        num_args: 1,
        primitive: true,
        handler: Some(mclass_handler),
        ..Default::default()
    }
}

fn mclass_handler(
    _parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    let body = require_function_arg(args, 0, &context.func_name)?;
    Ok(ParseNode::MClass {
        mode: context.mode,
        mclass: command_mclass(&context.func_name)?,
        body: ord_argument(body.clone()),
        is_character_box: is_character_box(&body),
    })
}

pub(crate) fn binrel_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec!["\\@binrel".to_string()],
        num_args: 2,
        handler: Some(binrel_handler),
        ..Default::default()
    }
}

fn binrel_handler(
    _parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    let class_arg = require_function_arg(args, 0, &context.func_name)?;
    let body = require_function_arg(args, 1, &context.func_name)?;
    Ok(ParseNode::MClass {
        mode: context.mode,
        mclass: binrel_class(&class_arg),
        body: ord_argument(body.clone()),
        is_character_box: is_character_box(&body),
    })
}

pub(crate) fn stackrel_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec![
            "\\stackrel".to_string(),
            "\\overset".to_string(),
            "\\underset".to_string(),
        ],
        num_args: 2,
        handler: Some(stackrel_handler),
        ..Default::default()
    }
}

fn stackrel_handler(
    _parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    let shifted = require_function_arg(args, 0, &context.func_name)?;
    let base_arg = require_function_arg(args, 1, &context.func_name)?;
    let mclass = if context.func_name == "\\stackrel" {
        AtomFamily::Mrel
    } else {
        binrel_class(&base_arg)
    };
    let base = ParseNode::Op {
        mode: base_arg.mode(),
        limits: true,
        always_handle_sup_sub: true,
        parent_is_sup_sub: false,
        suppress_base_shift: context.func_name != "\\stackrel",
        content: crate::ast::OperatorContent::BodyOperator(ord_argument(base_arg)),
    };
    let stacked = if context.func_name == "\\underset" {
        ParseNode::SupSub {
            mode: shifted.mode(),
            base: Some(Box::new(base)),
            sup: None,
            sub: Some(Box::new(shifted)),
        }
    } else {
        ParseNode::SupSub {
            mode: shifted.mode(),
            base: Some(Box::new(base)),
            sup: Some(Box::new(shifted)),
            sub: None,
        }
    };
    Ok(ParseNode::MClass {
        mode: context.mode,
        mclass,
        body: vec![stacked.clone()],
        is_character_box: is_character_box(&stacked),
    })
}
