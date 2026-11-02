use crate::ast::ParseNode;
use crate::error::ParseError;
use crate::function_registry::{ArgType, FunctionContext, FunctionParser, FunctionSpec};
use crate::settings::TrustContext;

use super::{ord_argument, require_function_arg};

pub(crate) fn href_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec!["\\href".to_string()],
        num_args: 2,
        arg_types: vec![ArgType::UrlArg, ArgType::OriginalArg],
        allowed_in_text: true,
        handler: Some(href_handler),
        ..Default::default()
    }
}

pub(crate) fn url_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec!["\\url".to_string()],
        num_args: 1,
        arg_types: vec![ArgType::UrlArg],
        allowed_in_text: true,
        handler: Some(url_handler),
        ..Default::default()
    }
}

fn href_argument(args: &[ParseNode], index: usize, func_name: &str) -> Result<String, ParseError> {
    match require_function_arg(args, index, func_name)? {
        ParseNode::Url { url, .. } => Ok(url),
        _ => Err(ParseError::InternalInvariant {
            message: format!("Expected URL argument for {func_name}"),
        }),
    }
}

fn href_handler(
    parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    let href = href_argument(args, 0, &context.func_name)?;
    let body = require_function_arg(args, 1, &context.func_name)?;
    if parser.is_trusted(TrustContext::UrlTrust {
        command: "\\href".to_string(),
        url: href.clone(),
        protocol: None,
    }) {
        Ok(ParseNode::Href {
            mode: context.mode,
            href,
            body: ord_argument(body),
        })
    } else {
        Ok(parser.unsupported_command(context.mode, &context.func_name, body))
    }
}

fn url_handler(
    parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    let href = href_argument(args, 0, &context.func_name)?;
    let chars: Vec<ParseNode> = href
        .chars()
        .map(|c| ParseNode::TextOrd {
            mode: crate::ast::Mode::Text,
            loc: None,
            text: if c == '~' {
                "\\textasciitilde".to_string()
            } else {
                c.to_string()
            },
        })
        .collect();
    let body = ParseNode::Text {
        mode: context.mode,
        font: "\\texttt".to_string(),
        body: chars,
    };
    if parser.is_trusted(TrustContext::UrlTrust {
        command: "\\url".to_string(),
        url: href.clone(),
        protocol: None,
    }) {
        Ok(ParseNode::Href {
            mode: context.mode,
            href,
            body: ord_argument(body.clone()),
        })
    } else {
        Ok(parser.unsupported_command(context.mode, &context.func_name, body))
    }
}
