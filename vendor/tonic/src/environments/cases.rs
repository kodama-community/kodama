use crate::ast::{ArrayColumn, ParseNode, StyleLevel};
use crate::environments::registry::{
    ArrayEnvironmentOptions, EnvironmentContext, EnvironmentParser,
};
use crate::error::ParseError;

fn cases_columns() -> Vec<ArrayColumn> {
    vec![
        ArrayColumn::AlignColumn {
            alignment: "l".to_string(),
            pre_gap: Some(0.0),
            post_gap: Some(1.0),
        },
        ArrayColumn::AlignColumn {
            alignment: "l".to_string(),
            pre_gap: Some(0.0),
            post_gap: Some(0.0),
        },
    ]
}

pub(crate) fn cases_environment_handler(
    parser: &mut dyn EnvironmentParser,
    context: &EnvironmentContext,
    _args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    let array = parser.parse_array(ArrayEnvironmentOptions {
        columns: Some(cases_columns()),
        array_stretch: Some(1.2),
        hskip_before_and_after: false,
        cell_style: if context.env_name == "dcases" || context.env_name == "drcases" {
            StyleLevel::DisplayStyle
        } else {
            StyleLevel::TextStyle
        },
        max_columns: None,
        single_row: false,
        empty_single_row: false,
        auto_tag: None,
        leqno: false,
        add_jot: false,
        column_separation_type: None,
    })?;
    let right_brace = context.env_name == "rcases" || context.env_name == "drcases";
    Ok(ParseNode::LeftRight {
        mode: context.mode,
        body: vec![array],
        left: if right_brace { "." } else { "\\{" }.to_string(),
        right: if right_brace { "\\}" } else { "." }.to_string(),
        right_color: None,
    })
}
