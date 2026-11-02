use crate::ast::{Measurement, Mode, ParseNode};
use crate::error::ParseError;
use crate::function_registry::{ArgType, FunctionContext, FunctionParser, FunctionSpec};
use crate::settings::TrustContext;

use super::{
    parse_decimal, parse_size_measurement, require_function_arg, trim_ascii_spaces, valid_size_unit,
};

pub(crate) fn includegraphics_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec!["\\includegraphics".to_string()],
        num_args: 1,
        num_optional_args: 1,
        arg_types: vec![ArgType::RawArg, ArgType::UrlArg],
        handler: Some(includegraphics_handler),
        ..Default::default()
    }
}

fn graphics_number(text: &str) -> Option<String> {
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut index = 0;
    let mut sign = String::new();
    if index < len && (chars[index] == '+' || chars[index] == '-') {
        sign.push(chars[index]);
        index = 1;
        while index < len && chars[index] == ' ' {
            index += 1;
        }
    }
    let start = index;
    while index < len && chars[index].is_ascii_digit() {
        index += 1;
    }
    let before = index - start;
    let mut after = 0;
    if index < len && chars[index] == '.' {
        index += 1;
        let fraction_start = index;
        while index < len && chars[index].is_ascii_digit() {
            index += 1;
        }
        after = index - fraction_start;
    }
    if index != len || (before == 0 && after == 0) {
        None
    } else {
        let tail: String = chars[start..].iter().collect();
        Some(sign + &tail)
    }
}

fn graphics_size(text: &str) -> Result<Measurement, ParseError> {
    let text = trim_ascii_spaces(text);
    if let Some(number_text) = graphics_number(text) {
        return Ok(Measurement {
            number: parse_decimal(&number_text)?,
            unit: "bp".to_string(),
        });
    }
    match parse_size_measurement(text)? {
        Some(size) if valid_size_unit(&size.unit) => Ok(size),
        Some(size) => Err(ParseError::InvalidArgument {
            message: format!("Invalid unit: '{}' in \\includegraphics.", size.unit),
            loc: None,
        }),
        None => Err(ParseError::InvalidArgument {
            message: format!("Invalid size: '{text}' in \\includegraphics"),
            loc: None,
        }),
    }
}

fn graphics_option_pair(item: &str) -> Option<(&str, &str)> {
    let (key, value) = item.split_once('=')?;
    if value.contains('=') {
        return None;
    }
    Some((trim_ascii_spaces(key), trim_ascii_spaces(value)))
}

fn graphics_default_alt(src: &str) -> String {
    src.rsplit(['/', '\\'])
        .next()
        .and_then(|name| name.rsplit_once('.'))
        .map_or_else(String::new, |(stem, _)| stem.to_string())
}

fn parse_graphics_options(
    value: &str,
) -> Result<(Measurement, Measurement, Measurement, String), ParseError> {
    let mut width = Measurement {
        number: 0.0,
        unit: "em".to_string(),
    };
    let mut height = Measurement {
        number: 0.9,
        unit: "em".to_string(),
    };
    let mut totalheight = Measurement {
        number: 0.0,
        unit: "em".to_string(),
    };
    let mut alt = String::new();
    for item in value.split(',') {
        let Some((key, value)) = graphics_option_pair(item) else {
            continue;
        };
        match key {
            "alt" => alt = value.to_string(),
            "width" => width = graphics_size(value)?,
            "height" => height = graphics_size(value)?,
            "totalheight" => totalheight = graphics_size(value)?,
            _ => {
                return Err(ParseError::InvalidArgument {
                    message: format!("Invalid key: '{key}' in \\includegraphics."),
                    loc: None,
                });
            }
        }
    }
    Ok((width, height, totalheight, alt))
}

fn includegraphics_handler(
    parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    let (width, height, totalheight, mut alt) = match opt_args.first().and_then(|o| o.as_ref()) {
        Some(ParseNode::Raw { string, .. }) => parse_graphics_options(string)?,
        Some(_) => {
            return Err(ParseError::InternalInvariant {
                message: "Expected raw graphics options".to_string(),
            });
        }
        None => parse_graphics_options("")?,
    };
    let src = match require_function_arg(args, 0, &context.func_name)? {
        ParseNode::Url { url, .. } => url,
        _ => {
            return Err(ParseError::InternalInvariant {
                message: "Expected URL argument for \\includegraphics".to_string(),
            });
        }
    };
    if alt.is_empty() {
        alt = graphics_default_alt(&src);
    }
    if parser.is_trusted(TrustContext::UrlTrust {
        command: "\\includegraphics".to_string(),
        url: src.clone(),
        protocol: None,
    }) {
        Ok(ParseNode::IncludeGraphics {
            mode: context.mode,
            alt,
            width,
            height,
            totalheight,
            src,
        })
    } else {
        Ok(parser.unsupported_command(
            context.mode,
            &context.func_name,
            ParseNode::TextOrd {
                mode: Mode::Text,
                loc: None,
                text: alt,
            },
        ))
    }
}
