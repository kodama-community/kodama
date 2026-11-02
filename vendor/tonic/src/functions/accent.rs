use crate::ast::ParseNode;
use crate::error::ParseError;
use crate::function_registry::{ArgType, FunctionContext, FunctionParser, FunctionSpec};

use super::{normalize_argument, require_function_arg};

pub(crate) fn accent_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec![
            "\\acute".to_string(),
            "\\grave".to_string(),
            "\\ddot".to_string(),
            "\\tilde".to_string(),
            "\\bar".to_string(),
            "\\breve".to_string(),
            "\\check".to_string(),
            "\\hat".to_string(),
            "\\vec".to_string(),
            "\\dot".to_string(),
            "\\mathring".to_string(),
            "\\widecheck".to_string(),
            "\\widehat".to_string(),
            "\\widetilde".to_string(),
            "\\overrightarrow".to_string(),
            "\\overleftarrow".to_string(),
            "\\Overrightarrow".to_string(),
            "\\overleftrightarrow".to_string(),
            "\\overgroup".to_string(),
            "\\overlinesegment".to_string(),
            "\\overleftharpoon".to_string(),
            "\\overrightharpoon".to_string(),
        ],
        num_args: 1,
        handler: Some(accent_handler),
        ..Default::default()
    }
}

fn is_non_stretchy_accent(func_name: &str) -> bool {
    matches!(
        func_name,
        "\\acute"
            | "\\grave"
            | "\\ddot"
            | "\\tilde"
            | "\\bar"
            | "\\breve"
            | "\\check"
            | "\\hat"
            | "\\vec"
            | "\\dot"
            | "\\mathring"
    )
}

fn accent_handler(
    _parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    let is_stretchy = !is_non_stretchy_accent(&context.func_name);
    Ok(ParseNode::Accent {
        mode: context.mode,
        loc: None,
        label: context.func_name.clone(),
        is_stretchy,
        is_shifty: !is_stretchy
            || context.func_name == "\\widehat"
            || context.func_name == "\\widetilde"
            || context.func_name == "\\widecheck",
        base: Box::new(normalize_argument(require_function_arg(
            args,
            0,
            &context.func_name,
        )?)),
    })
}

pub(crate) fn text_accent_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec![
            "\\'".to_string(),
            "\\`".to_string(),
            "\\^".to_string(),
            "\\~".to_string(),
            "\\=".to_string(),
            "\\u".to_string(),
            "\\.".to_string(),
            "\\\"".to_string(),
            "\\c".to_string(),
            "\\r".to_string(),
            "\\H".to_string(),
            "\\v".to_string(),
            "\\textcircled".to_string(),
        ],
        num_args: 1,
        arg_types: vec![ArgType::PrimitiveArg],
        allowed_in_text: true,
        handler: Some(text_accent_handler),
        ..Default::default()
    }
}

fn text_accent_handler(
    parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    let mode = if context.mode == crate::ast::Mode::Math {
        parser.report_nonstrict(
            "mathVsTextAccents",
            &format!(
                "LaTeX's accent {} works only in text mode",
                context.func_name
            ),
            context.token.as_ref(),
        )?;
        crate::ast::Mode::Text
    } else {
        context.mode
    };
    Ok(ParseNode::Accent {
        mode,
        loc: None,
        label: context.func_name.clone(),
        is_stretchy: false,
        is_shifty: true,
        base: Box::new(require_function_arg(args, 0, &context.func_name)?),
    })
}
