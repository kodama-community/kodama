use crate::ast::ParseNode;
use crate::error::ParseError;
use crate::function_registry::{FunctionContext, FunctionParser, FunctionSpec};

use super::require_function_arg;

pub(crate) fn smash_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec!["\\smash".to_string()],
        num_args: 1,
        num_optional_args: 1,
        allowed_in_text: true,
        handler: Some(smash_handler),
        ..Default::default()
    }
}

fn smash_flags(arg: Option<&ParseNode>) -> Result<(bool, bool), ParseError> {
    let Some(ParseNode::OrdGroup { body, .. }) = arg else {
        return Ok((true, true));
    };
    let mut smash_height = false;
    let mut smash_depth = false;
    for node in body {
        let text = match node {
            ParseNode::MathOrd { text, .. }
            | ParseNode::TextOrd { text, .. }
            | ParseNode::Atom { text, .. } => text,
            _ => {
                return Err(ParseError::InternalInvariant {
                    message: "Expected symbol in \\smash option".to_string(),
                });
            }
        };
        if text == "t" {
            smash_height = true;
        } else if text == "b" {
            smash_depth = true;
        } else {
            return Ok((false, false));
        }
    }
    Ok((smash_height, smash_depth))
}

fn smash_handler(
    _parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    let (smash_height, smash_depth) = smash_flags(opt_args.first().and_then(|o| o.as_ref()))?;
    Ok(ParseNode::Smash {
        mode: context.mode,
        body: Box::new(require_function_arg(args, 0, &context.func_name)?),
        smash_height,
        smash_depth,
    })
}
