use crate::ast::{ParseNode, StyleLevel};
use crate::environments::registry::{
    ArrayEnvironmentOptions, EnvironmentContext, EnvironmentParser,
};
use crate::error::ParseError;

pub(crate) fn equation_environment_handler(
    parser: &mut dyn EnvironmentParser,
    context: &EnvironmentContext,
    _args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    if !context.display_mode {
        return Err(ParseError::InvalidArgument {
            message: format!("{{{}}} can be used only in display mode.", context.env_name),
            loc: None,
        });
    }
    parser.parse_array(ArrayEnvironmentOptions {
        columns: None,
        array_stretch: None,
        hskip_before_and_after: false,
        cell_style: StyleLevel::DisplayStyle,
        max_columns: Some(1),
        single_row: true,
        empty_single_row: true,
        auto_tag: Some(context.env_name == "equation"),
        leqno: context.leqno,
        add_jot: false,
        column_separation_type: None,
    })
}
