use crate::ast::ParseNode;
use crate::error::ParseError;
use crate::function_registry::{FunctionContext, FunctionParser, FunctionSpec};
use crate::macro_definition::{MacroDefinition, MacroExpansion};
use crate::token::Token;

fn internal_node(context: &FunctionContext) -> ParseNode {
    ParseNode::Internal { mode: context.mode }
}

fn check_control_sequence(token: &Token) -> Result<String, ParseError> {
    match token.text.as_str() {
        "\\" | "{" | "}" | "$" | "&" | "#" | "^" | "_" | "EOF" => {
            Err(ParseError::InvalidArgument {
                message: "Expected a control sequence".to_string(),
                loc: token.loc.clone(),
            })
        }
        name => Ok(name.to_string()),
    }
}

pub(crate) fn macro_prefix_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec![
            "\\global".to_string(),
            "\\long".to_string(),
            "\\\\globallong".to_string(),
        ],
        allowed_in_text: true,
        handler: Some(macro_prefix_handler),
        ..Default::default()
    }
}

fn prefixed_macro_name(prefix: &str, next: &str) -> Option<String> {
    let global = prefix == "\\global" || prefix == "\\\\globallong";
    let name = match next {
        "\\global" => "\\global",
        "\\long" => {
            if global {
                "\\\\globallong"
            } else {
                "\\long"
            }
        }
        "\\\\globallong" => "\\\\globallong",
        "\\def" => {
            if global {
                "\\gdef"
            } else {
                "\\def"
            }
        }
        "\\gdef" => "\\gdef",
        "\\edef" => {
            if global {
                "\\xdef"
            } else {
                "\\edef"
            }
        }
        "\\xdef" => "\\xdef",
        "\\let" => {
            if global {
                "\\\\globallet"
            } else {
                "\\let"
            }
        }
        "\\futurelet" => {
            if global {
                "\\\\globalfuture"
            } else {
                "\\futurelet"
            }
        }
        _ => return None,
    };
    Some(name.to_string())
}

fn macro_prefix_handler(
    parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    _args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    parser.consume_spaces()?;
    let token = parser.pop_token()?;
    let Some(name) = prefixed_macro_name(&context.func_name, &token.text) else {
        return Err(ParseError::InvalidArgument {
            message: "Invalid token after macro prefix".to_string(),
            loc: token.loc.clone(),
        });
    };
    parser.parse_prefixed_function(&name)
}

pub(crate) fn definition_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec![
            "\\def".to_string(),
            "\\gdef".to_string(),
            "\\edef".to_string(),
            "\\xdef".to_string(),
        ],
        allowed_in_text: true,
        primitive: true,
        handler: Some(definition_handler),
        ..Default::default()
    }
}

fn definition_handler(
    parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    _args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    let name = check_control_sequence(&parser.pop_token()?)?;
    let (delimiters, count, insert) = scan_definition_parameters(parser)?;
    let tokens = parser.consume_macro_arg()?;
    let tokens = expand_definition_body(parser, context, tokens, insert)?;
    parser.set_macro_definition(
        &name,
        MacroDefinition::expansion(MacroExpansion {
            tokens,
            num_args: count,
            delimiters: Some(delimiters),
            unexpandable: false,
        }),
        context.func_name == "\\gdef" || context.func_name == "\\xdef",
    );
    Ok(internal_node(context))
}

type DefinitionParameters = (Vec<Vec<String>>, usize, Option<Token>);

fn scan_definition_parameters(
    parser: &mut dyn FunctionParser,
) -> Result<DefinitionParameters, ParseError> {
    let mut delimiters: Vec<Vec<String>> = vec![Vec::new()];
    let mut count = 0;
    let mut insert: Option<Token> = None;
    while parser.future_token()?.text != "{" {
        let token = parser.pop_token()?;
        if token.text == "#" {
            if parser.future_token()?.text == "{" {
                let brace = parser.future_token()?;
                delimiters[count].push("{".to_string());
                insert = Some(brace);
                break;
            }
            let number = parser.pop_token()?;
            if !is_definition_arg_number(&number) {
                return Err(ParseError::InvalidArgument {
                    message: format!("Invalid argument number \"{}\"", number.text),
                    loc: number.loc.clone(),
                });
            }
            let digit = number.text.chars().next().unwrap() as u32 - '0' as u32;
            if digit as usize != count + 1 {
                return Err(ParseError::InvalidArgument {
                    message: format!("Argument number \"{}\" out of order", number.text),
                    loc: number.loc.clone(),
                });
            }
            count += 1;
            delimiters.push(Vec::new());
        } else if token.text == "EOF" {
            return Err(ParseError::InvalidArgument {
                message: "Expected a macro definition".to_string(),
                loc: token.loc.clone(),
            });
        } else {
            delimiters[count].push(token.text);
        }
    }
    Ok((delimiters, count, insert))
}

fn is_definition_arg_number(token: &Token) -> bool {
    token.text.len() == 1
        && token
            .text
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_digit() && c >= '1')
}

fn expand_definition_body(
    parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    tokens: Vec<Token>,
    insert: Option<Token>,
) -> Result<Vec<Token>, ParseError> {
    let tokens = if let Some(token) = insert {
        let mut combined = vec![token];
        combined.extend(tokens);
        combined
    } else {
        tokens
    };
    if context.func_name == "\\edef" || context.func_name == "\\xdef" {
        let mut expanded = parser.expand_tokens(tokens)?;
        expanded.reverse();
        Ok(expanded)
    } else {
        Ok(tokens)
    }
}

pub(crate) fn let_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec!["\\let".to_string(), "\\\\globallet".to_string()],
        allowed_in_text: true,
        primitive: true,
        handler: Some(let_handler),
        ..Default::default()
    }
}

fn let_rhs(parser: &mut dyn FunctionParser) -> Result<Token, ParseError> {
    let mut token = parser.pop_token()?;
    if token.text == "=" {
        token = parser.pop_token()?;
        if token.text == " " {
            token = parser.pop_token()?;
        }
    }
    Ok(token)
}

fn assign_let(parser: &mut dyn FunctionParser, name: &str, mut token: Token, global: bool) {
    let definition = match parser.get_macro(&token.text) {
        Some(value) => value,
        None => {
            let unexpandable = !parser.is_expandable(&token.text);
            token.noexpand = true;
            MacroDefinition::expansion(MacroExpansion {
                tokens: vec![token],
                num_args: 0,
                delimiters: None,
                unexpandable,
            })
        }
    };
    parser.set_macro_definition(name, definition, global);
}

fn let_handler(
    parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    _args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    let name = check_control_sequence(&parser.pop_token()?)?;
    parser.consume_spaces()?;
    let token = let_rhs(parser)?;
    assign_let(parser, &name, token, context.func_name == "\\\\globallet");
    Ok(internal_node(context))
}

pub(crate) fn futurelet_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec!["\\futurelet".to_string(), "\\\\globalfuture".to_string()],
        allowed_in_text: true,
        primitive: true,
        handler: Some(futurelet_handler),
        ..Default::default()
    }
}

fn futurelet_handler(
    parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    _args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    let name = check_control_sequence(&parser.pop_token()?)?;
    let middle = parser.pop_token()?;
    let token = parser.pop_token()?;
    assign_let(
        parser,
        &name,
        token.clone(),
        context.func_name == "\\\\globalfuture",
    );
    parser.push_token(token);
    parser.push_token(middle);
    Ok(internal_node(context))
}
