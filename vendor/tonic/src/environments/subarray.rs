use crate::ast::{ArrayColumn, ColumnSeparationType, ParseNode, StyleLevel};
use crate::environments::registry::{
    ArrayEnvironmentOptions, EnvironmentContext, EnvironmentParser, array_columns,
};
use crate::error::ParseError;
use crate::functions::require_function_arg;

fn subarray_columns(arg: &ParseNode) -> Result<Vec<ArrayColumn>, ParseError> {
    let columns = array_columns(arg, "subarray")?;
    if columns.len() > 1 {
        return Err(ParseError::InvalidArgument {
            message: "{subarray} can contain only one column".to_string(),
            loc: None,
        });
    }
    match columns.as_slice() {
        [] => Ok(columns),
        [ArrayColumn::AlignColumn { alignment, .. }] if alignment == "l" || alignment == "c" => {
            Ok(columns)
        }
        _ => Err(ParseError::InvalidArgument {
            message: "Unknown column alignment in subarray".to_string(),
            loc: None,
        }),
    }
}

pub(crate) fn subarray_environment_handler(
    parser: &mut dyn EnvironmentParser,
    _context: &EnvironmentContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    parser.parse_array(ArrayEnvironmentOptions {
        columns: Some(subarray_columns(&require_function_arg(
            args,
            0,
            "\\begin{subarray}",
        )?)?),
        array_stretch: Some(0.5),
        hskip_before_and_after: false,
        cell_style: StyleLevel::ScriptStyle,
        max_columns: Some(1),
        single_row: false,
        empty_single_row: false,
        auto_tag: None,
        leqno: false,
        add_jot: false,
        column_separation_type: Some(ColumnSeparationType::SmallSeparation),
    })
}
