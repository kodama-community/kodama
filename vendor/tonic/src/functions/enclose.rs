use crate::ast::ParseNode;
use crate::error::ParseError;
use crate::function_registry::{ArgType, FunctionContext, FunctionParser, FunctionSpec};

use super::{require_color_argument, require_color_argument_at, require_function_arg};

pub(crate) fn colorbox_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec!["\\colorbox".to_string()],
        num_args: 2,
        arg_types: vec![ArgType::ColorArg, ArgType::HboxArg],
        allowed_in_text: true,
        handler: Some(colorbox_handler),
        ..Default::default()
    }
}

fn colorbox_handler(
    _parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    Ok(ParseNode::Enclose {
        mode: context.mode,
        label: context.func_name.clone(),
        background_color: Some(require_color_argument(args, &context.func_name)?),
        border_color: None,
        body: Box::new(require_function_arg(args, 1, &context.func_name)?),
    })
}

pub(crate) fn fcolorbox_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec!["\\fcolorbox".to_string()],
        num_args: 3,
        arg_types: vec![ArgType::ColorArg, ArgType::ColorArg, ArgType::HboxArg],
        allowed_in_text: true,
        handler: Some(fcolorbox_handler),
        ..Default::default()
    }
}

fn fcolorbox_handler(
    _parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    Ok(ParseNode::Enclose {
        mode: context.mode,
        label: context.func_name.clone(),
        border_color: Some(require_color_argument_at(args, 0, &context.func_name)?),
        background_color: Some(require_color_argument_at(args, 1, &context.func_name)?),
        body: Box::new(require_function_arg(args, 2, &context.func_name)?),
    })
}

pub(crate) fn fbox_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec!["\\fbox".to_string()],
        num_args: 1,
        arg_types: vec![ArgType::HboxArg],
        allowed_in_text: true,
        handler: Some(enclose_handler),
        ..Default::default()
    }
}

pub(crate) fn cancel_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec![
            "\\cancel".to_string(),
            "\\bcancel".to_string(),
            "\\xcancel".to_string(),
            "\\phase".to_string(),
        ],
        num_args: 1,
        handler: Some(enclose_handler),
        ..Default::default()
    }
}

pub(crate) fn sout_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec!["\\sout".to_string()],
        num_args: 1,
        allowed_in_text: true,
        handler: Some(sout_handler),
        ..Default::default()
    }
}

fn sout_handler(
    parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    if context.mode == crate::ast::Mode::Math {
        parser.report_nonstrict(
            "mathVsSout",
            "LaTeX's \\sout works only in text mode",
            context.token.as_ref(),
        )?;
    }
    enclose_node(context, args, 0)
}

pub(crate) fn angl_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec!["\\angl".to_string()],
        num_args: 1,
        arg_types: vec![ArgType::HboxArg],
        handler: Some(enclose_handler),
        ..Default::default()
    }
}

fn enclose_handler(
    _parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    enclose_node(context, args, 0)
}

fn enclose_node(
    context: &FunctionContext,
    args: &[ParseNode],
    body_index: usize,
) -> Result<ParseNode, ParseError> {
    Ok(ParseNode::Enclose {
        mode: context.mode,
        label: context.func_name.clone(),
        background_color: None,
        border_color: None,
        body: Box::new(require_function_arg(args, body_index, &context.func_name)?),
    })
}
