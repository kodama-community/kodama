use crate::ast::{ColumnSeparationType, ParseNode, StyleLevel};
use crate::environments::registry::{
    ArrayEnvironmentOptions, EnvironmentContext, EnvironmentParser,
};
use crate::error::ParseError;
use crate::functions::require_function_arg;

pub(crate) fn environment_argument_text(arg: &ParseNode) -> Result<String, ParseError> {
    let nodes = match arg {
        ParseNode::OrdGroup { body, .. } => body,
        _ => std::slice::from_ref(arg),
    };
    let mut text = String::new();
    for node in nodes {
        let part = match node {
            ParseNode::MathOrd { text, .. }
            | ParseNode::TextOrd { text, .. }
            | ParseNode::Atom { text, .. } => text.as_str(),
            _ => {
                return Err(ParseError::InvalidArgument {
                    message: "Invalid alignment column count".to_string(),
                    loc: None,
                });
            }
        };
        text.push_str(part);
    }
    Ok(text)
}

fn alignat_pair_count(arg: &ParseNode) -> Result<usize, ParseError> {
    let text = environment_argument_text(arg)?;
    let Some(count) = crate::builtin_macros_commands::parse_argument_count(&text) else {
        return Err(ParseError::InvalidArgument {
            message: "Invalid alignment column count".to_string(),
            loc: None,
        });
    };
    Ok(count)
}

pub(crate) fn alignat_environment_handler(
    parser: &mut dyn EnvironmentParser,
    context: &EnvironmentContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    if context.env_name != "alignedat" && !context.display_mode {
        return Err(ParseError::InvalidArgument {
            message: format!("{{{}}} can be used only in display mode.", context.env_name),
            loc: None,
        });
    }
    let pairs = alignat_pair_count(&require_function_arg(
        args,
        0,
        &format!("\\begin{{{}}}", context.env_name),
    )?)?;
    let auto_tag = match context.env_name.as_str() {
        "alignat" => Some(true),
        "alignat*" => Some(false),
        _ => None,
    };
    let array = parser.parse_array(ArrayEnvironmentOptions {
        columns: None,
        array_stretch: None,
        hskip_before_and_after: false,
        cell_style: StyleLevel::DisplayStyle,
        max_columns: Some(pairs * 2),
        single_row: false,
        empty_single_row: true,
        auto_tag,
        leqno: context.leqno,
        add_jot: true,
        column_separation_type: Some(ColumnSeparationType::AlignAtSeparation),
    })?;
    let array = super::alignment::insert_alignment_empty_groups(array)?;
    let count = super::alignment::array_body(&array)?
        .iter()
        .map(Vec::len)
        .max()
        .unwrap_or(0);
    super::alignment::replace_alignment_columns(
        array,
        super::alignment::alignment_columns(count, false),
    )
}
