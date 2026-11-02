use crate::ast::{Measurement, ParseNode};
use crate::error::ParseError;
use crate::function_registry::{ArgType, FunctionContext, FunctionParser, FunctionSpec};

use super::require_function_arg;

pub(crate) fn kern_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec![
            "\\kern".to_string(),
            "\\mkern".to_string(),
            "\\hskip".to_string(),
            "\\mskip".to_string(),
        ],
        num_args: 1,
        arg_types: vec![ArgType::SizeArg],
        primitive: true,
        allowed_in_text: true,
        handler: Some(kern_handler),
        ..Default::default()
    }
}

fn kern_dimension(args: &[ParseNode], func_name: &str) -> Result<Measurement, ParseError> {
    match require_function_arg(args, 0, func_name)? {
        ParseNode::Size { value, .. } => Ok(value),
        _ => Err(ParseError::InternalInvariant {
            message: format!("Expected size argument for {func_name}"),
        }),
    }
}

fn is_math_spacing_function(func_name: &str) -> bool {
    func_name == "\\mkern" || func_name == "\\mskip"
}

fn kern_handler(
    parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    let dimension = kern_dimension(args, &context.func_name)?;
    let uses_mu = dimension.unit == "mu";
    if is_math_spacing_function(&context.func_name) {
        if !uses_mu {
            parser.report_nonstrict(
                "mathVsTextUnits",
                &format!(
                    "LaTeX's {} supports only mu units, not {} units",
                    context.func_name, dimension.unit
                ),
                context.token.as_ref(),
            )?;
        }
        if context.mode != crate::ast::Mode::Math {
            parser.report_nonstrict(
                "mathVsTextUnits",
                &format!("LaTeX's {} works only in math mode", context.func_name),
                context.token.as_ref(),
            )?;
        }
    } else if uses_mu {
        parser.report_nonstrict(
            "mathVsTextUnits",
            &format!("LaTeX's {} doesn't support mu units", context.func_name),
            context.token.as_ref(),
        )?;
    }
    Ok(ParseNode::Kern {
        mode: context.mode,
        dimension,
    })
}
