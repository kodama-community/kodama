use crate::ast::{Mode, OperatorContent, ParseNode};
use crate::error::ParseError;
use crate::function_registry::{FunctionContext, FunctionParser, FunctionSpec};

use super::{ord_argument, require_function_arg};

pub(crate) const BIG_OPERATOR_COMMANDS: &[(&str, &str)] = &[
    ("\\coprod", "\\coprod"),
    ("\\bigvee", "\\bigvee"),
    ("\\bigwedge", "\\bigwedge"),
    ("\\biguplus", "\\biguplus"),
    ("\\bigcap", "\\bigcap"),
    ("\\bigcup", "\\bigcup"),
    ("\\intop", "\\intop"),
    ("\\prod", "\\prod"),
    ("\\sum", "\\sum"),
    ("\\bigotimes", "\\bigotimes"),
    ("\\bigoplus", "\\bigoplus"),
    ("\\bigodot", "\\bigodot"),
    ("\\bigsqcup", "\\bigsqcup"),
    ("\\smallint", "\\smallint"),
    ("∏", "\\prod"),
    ("∐", "\\coprod"),
    ("∑", "\\sum"),
    ("⋀", "\\bigwedge"),
    ("⋁", "\\bigvee"),
    ("⋂", "\\bigcap"),
    ("⋃", "\\bigcup"),
    ("⨀", "\\bigodot"),
    ("⨁", "\\bigoplus"),
    ("⨂", "\\bigotimes"),
    ("⨄", "\\biguplus"),
    ("⨆", "\\bigsqcup"),
];

pub(crate) const INTEGRAL_OPERATOR_COMMANDS: &[(&str, &str)] = &[
    ("\\int", "\\int"),
    ("\\iint", "\\iint"),
    ("\\iiint", "\\iiint"),
    ("\\oint", "\\oint"),
    ("\\oiint", "\\oiint"),
    ("\\oiiint", "\\oiiint"),
    ("∫", "\\int"),
    ("∬", "\\iint"),
    ("∭", "\\iiint"),
    ("∮", "\\oint"),
    ("∯", "\\oiint"),
    ("∰", "\\oiiint"),
];

pub(crate) const NAMED_OPERATOR_COMMANDS: &[&str] = &[
    "\\arcsin", "\\arccos", "\\arctan", "\\arctg", "\\arcctg", "\\arg", "\\ch", "\\cos", "\\cosec",
    "\\cosh", "\\cot", "\\cotg", "\\coth", "\\csc", "\\ctg", "\\cth", "\\deg", "\\dim", "\\exp",
    "\\hom", "\\ker", "\\lg", "\\ln", "\\log", "\\sec", "\\sin", "\\sinh", "\\sh", "\\tan",
    "\\tanh", "\\tg", "\\th",
];

pub(crate) const LIMITED_NAMED_OPERATOR_COMMANDS: &[&str] = &[
    "\\det", "\\gcd", "\\inf", "\\lim", "\\max", "\\min", "\\Pr", "\\sup",
];

fn operator_names(commands: &[(&str, &str)]) -> Vec<String> {
    commands.iter().map(|(name, _)| name.to_string()).collect()
}

fn canonical_operator_name(
    commands: &[(&str, &str)],
    func_name: &str,
) -> Result<String, ParseError> {
    for (name, canonical) in commands {
        if *name == func_name {
            return Ok((*canonical).to_string());
        }
    }
    Err(ParseError::InternalInvariant {
        message: format!("Unknown operator command: {func_name}"),
    })
}

fn symbol_operator(mode: Mode, limits: bool, name: String) -> ParseNode {
    ParseNode::Op {
        mode,
        limits,
        always_handle_sup_sub: false,
        parent_is_sup_sub: false,
        suppress_base_shift: false,
        content: OperatorContent::SymbolOperator(name),
    }
}

fn named_operator(mode: Mode, limits: bool, name: String) -> ParseNode {
    ParseNode::Op {
        mode,
        limits,
        always_handle_sup_sub: false,
        parent_is_sup_sub: false,
        suppress_base_shift: false,
        content: OperatorContent::NamedOperator(name),
    }
}

pub(crate) fn big_operator_spec() -> FunctionSpec {
    FunctionSpec {
        names: operator_names(BIG_OPERATOR_COMMANDS),
        handler: Some(big_operator_handler),
        ..Default::default()
    }
}

fn big_operator_handler(
    _parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    _args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    Ok(symbol_operator(
        context.mode,
        true,
        canonical_operator_name(BIG_OPERATOR_COMMANDS, &context.func_name)?,
    ))
}

pub(crate) fn mathop_spec() -> FunctionSpec {
    FunctionSpec {
        names: vec!["\\mathop".to_string()],
        num_args: 1,
        primitive: true,
        handler: Some(mathop_handler),
        ..Default::default()
    }
}

fn mathop_handler(
    _parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    Ok(ParseNode::Op {
        mode: context.mode,
        limits: false,
        always_handle_sup_sub: false,
        parent_is_sup_sub: false,
        suppress_base_shift: false,
        content: OperatorContent::BodyOperator(ord_argument(require_function_arg(
            args,
            0,
            &context.func_name,
        )?)),
    })
}

pub(crate) fn named_operator_spec() -> FunctionSpec {
    FunctionSpec {
        names: NAMED_OPERATOR_COMMANDS
            .iter()
            .map(ToString::to_string)
            .collect(),
        handler: Some(named_operator_handler),
        ..Default::default()
    }
}

fn named_operator_handler(
    _parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    _args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    Ok(named_operator(
        context.mode,
        false,
        context.func_name.clone(),
    ))
}

pub(crate) fn limited_named_operator_spec() -> FunctionSpec {
    FunctionSpec {
        names: LIMITED_NAMED_OPERATOR_COMMANDS
            .iter()
            .map(ToString::to_string)
            .collect(),
        handler: Some(limited_named_operator_handler),
        ..Default::default()
    }
}

fn limited_named_operator_handler(
    _parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    _args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    Ok(named_operator(
        context.mode,
        true,
        context.func_name.clone(),
    ))
}

pub(crate) fn integral_operator_spec() -> FunctionSpec {
    FunctionSpec {
        names: operator_names(INTEGRAL_OPERATOR_COMMANDS),
        allowed_in_argument: true,
        handler: Some(integral_operator_handler),
        ..Default::default()
    }
}

fn integral_operator_handler(
    _parser: &mut dyn FunctionParser,
    context: &FunctionContext,
    _args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    Ok(symbol_operator(
        context.mode,
        false,
        canonical_operator_name(INTEGRAL_OPERATOR_COMMANDS, &context.func_name)?,
    ))
}
