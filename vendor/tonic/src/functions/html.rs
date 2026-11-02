use crate::ast::ParseNode;
use crate::error::ParseError;
use crate::function_registry::{ArgType, FunctionContext, FunctionParser, FunctionSpec};
use crate::settings::TrustContext;

use super::{ord_argument, require_function_arg};

pub(crate) fn trim_ascii_spaces(text: &str) -> &str {
    let start = text
        .char_indices()
        .find(|(_, c)| !matches!(c, ' ' | '\t' | '\n' | '\r'))
        .map_or(text.len(), |(i, _)| i);
    let end = text
        .char_indices()
        .rev()
        .find(|(_, c)| !matches!(c, ' ' | '\t' | '\n' | '\r'))
        .map_or(start, |(i, c)| i + c.len_utf8());
    &text[start..end]
}

fn html_data_attributes(value: &str) -> Result<Vec<(String, String)>, ParseError> {
    let mut attributes: Vec<(String, String)> = Vec::new();
    let mut items = vec![String::new()];
    let mut remaining = value;
    while !remaining.is_empty() {
        if let Some(rest) = remaining.strip_prefix("{,}") {
            items.last_mut().unwrap().push(',');
            remaining = rest;
        } else {
            let ch = remaining.chars().next().unwrap();
            if ch == ',' {
                items.push(String::new());
            } else {
                items.last_mut().unwrap().push(ch);
            }
            remaining = &remaining[ch.len_utf8()..];
        }
    }
    for item in items {
        let Some(equals) = item.find('=') else {
            return Err(ParseError::InvalidArgument {
                message: format!("\\htmlData key/value '{item}' missing equals sign"),
                loc: None,
            });
        };
        let key = item[..equals].trim_matches(|ch: char| ch.is_whitespace() || ch == '\u{feff}');
        let data_value = item[equals + 1..].to_string();
        let key = format!("data-{key}");
        if let Some((_, value)) = attributes.iter_mut().find(|(name, _)| *name == key) {
            *value = data_value;
        } else {
            attributes.push((key, data_value));
        }
    }
    Ok(attributes)
}

pub(crate) fn html_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec![
            "\\htmlClass".to_string(),
            "\\htmlId".to_string(),
            "\\htmlStyle".to_string(),
            "\\htmlData".to_string(),
        ],
        num_args: 2,
        arg_types: vec![ArgType::RawArg, ArgType::OriginalArg],
        allowed_in_text: true,
        handler: Some(html_handler),
        ..Default::default()
    }
}

fn html_handler(
    parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    let value = match require_function_arg(args, 0, &context.func_name)? {
        ParseNode::Raw { string, .. } => string,
        _ => {
            return Err(ParseError::InternalInvariant {
                message: format!("Expected raw argument for {}", context.func_name),
            });
        }
    };
    parser.report_nonstrict(
        "htmlExtension",
        "HTML extension is disabled on strict mode",
        context.token.as_ref(),
    )?;
    let mut attributes: Vec<(String, String)> = Vec::new();
    let trust_context = match context.func_name.as_str() {
        "\\htmlClass" => {
            attributes.push(("class".to_string(), value.clone()));
            TrustContext::HtmlClass { class: value }
        }
        "\\htmlId" => {
            attributes.push(("id".to_string(), value.clone()));
            TrustContext::HtmlId { id: value }
        }
        "\\htmlStyle" => {
            attributes.push(("style".to_string(), value.clone()));
            TrustContext::HtmlStyle { style: value }
        }
        "\\htmlData" => {
            attributes = html_data_attributes(&value)?;
            TrustContext::HtmlData {
                attributes: attributes.iter().cloned().collect(),
            }
        }
        _ => {
            return Err(ParseError::InternalInvariant {
                message: "Unrecognized html command".to_string(),
            });
        }
    };
    let body = ord_argument(require_function_arg(args, 1, &context.func_name)?);
    if parser.is_trusted(trust_context) {
        Ok(ParseNode::Html {
            mode: context.mode,
            attributes,
            body,
        })
    } else {
        Ok(parser.unsupported_command(
            context.mode,
            &context.func_name,
            ParseNode::OrdGroup {
                mode: context.mode,
                loc: None,
                body,
                semisimple: false,
            },
        ))
    }
}
