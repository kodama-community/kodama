use crate::ast::{ArrayColumn, ColumnSeparationType, ParseNode, StyleLevel};
use crate::environments::registry::{
    ArrayEnvironmentOptions, EnvironmentContext, EnvironmentParser,
};
use crate::error::ParseError;

pub(crate) fn matrix_column_count(body: &[Vec<ParseNode>]) -> usize {
    body.iter().map(Vec::len).max().unwrap_or(0)
}

fn matrix_columns(count: usize, alignment: &str) -> Vec<ArrayColumn> {
    (0..count)
        .map(|_| ArrayColumn::AlignColumn {
            alignment: alignment.to_string(),
            pre_gap: None,
            post_gap: None,
        })
        .collect()
}

pub(crate) fn array_with_columns(
    node: ParseNode,
    columns: Vec<ArrayColumn>,
) -> Result<ParseNode, ParseError> {
    match node {
        ParseNode::Array {
            mode,
            body,
            add_jot,
            array_stretch,
            row_gaps,
            hskip_before_and_after,
            hlines_before_row,
            column_separation_type,
            tags,
            auto_tags,
            leqno,
            ..
        } => Ok(ParseNode::Array {
            mode,
            body,
            add_jot,
            array_stretch,
            columns: Some(columns),
            row_gaps,
            hskip_before_and_after,
            hlines_before_row,
            column_separation_type,
            tags,
            auto_tags,
            leqno,
        }),
        _ => Err(ParseError::InternalInvariant {
            message: "Expected matrix array".to_string(),
        }),
    }
}

fn matrix_delimiters(name: &str) -> Option<(String, String)> {
    match name {
        "matrix" => None,
        "pmatrix" => Some(("(".to_string(), ")".to_string())),
        "bmatrix" => Some(("[".to_string(), "]".to_string())),
        "Bmatrix" => Some(("\\{".to_string(), "\\}".to_string())),
        "vmatrix" => Some(("|".to_string(), "|".to_string())),
        "Vmatrix" => Some(("\\Vert".to_string(), "\\Vert".to_string())),
        _ => None,
    }
}

pub(crate) fn matrix_environment_handler(
    parser: &mut dyn EnvironmentParser,
    context: &EnvironmentContext,
    _args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    let starred = context.env_name.ends_with('*');
    let base_name = if starred {
        context.env_name[..context.env_name.len() - 1].to_string()
    } else {
        context.env_name.clone()
    };
    let requested_alignment = if starred {
        parser.parse_matrix_alignment()?
    } else {
        None
    };
    let alignment = requested_alignment.unwrap_or_else(|| "c".to_string());
    let array = parser.parse_array(ArrayEnvironmentOptions {
        columns: None,
        array_stretch: None,
        hskip_before_and_after: false,
        cell_style: StyleLevel::TextStyle,
        max_columns: None,
        single_row: false,
        empty_single_row: false,
        auto_tag: None,
        leqno: false,
        add_jot: false,
        column_separation_type: None,
    })?;
    let count = match &array {
        ParseNode::Array { body, .. } => matrix_column_count(body),
        _ => {
            return Err(ParseError::InternalInvariant {
                message: "Expected matrix array".to_string(),
            });
        }
    };
    let array = array_with_columns(array, matrix_columns(count, &alignment))?;
    if let Some((left, right)) = matrix_delimiters(&base_name) {
        Ok(ParseNode::LeftRight {
            mode: context.mode,
            body: vec![array],
            left,
            right,
            right_color: None,
        })
    } else {
        Ok(array)
    }
}

pub(crate) fn smallmatrix_environment_handler(
    parser: &mut dyn EnvironmentParser,
    _context: &EnvironmentContext,
    _args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    let array = parser.parse_array(ArrayEnvironmentOptions {
        columns: None,
        array_stretch: Some(0.5),
        hskip_before_and_after: false,
        cell_style: StyleLevel::ScriptStyle,
        max_columns: None,
        single_row: false,
        empty_single_row: false,
        auto_tag: None,
        leqno: false,
        add_jot: false,
        column_separation_type: Some(ColumnSeparationType::SmallSeparation),
    })?;
    let count = match &array {
        ParseNode::Array { body, .. } => matrix_column_count(body),
        _ => {
            return Err(ParseError::InternalInvariant {
                message: "Expected smallmatrix array".to_string(),
            });
        }
    };
    array_with_columns(array, matrix_columns(count, "c"))
}
