use crate::ast::{ArrayColumn, ColumnSeparationType, ParseNode, StyleLevel};
use crate::environments::registry::{
    ArrayEnvironmentOptions, EnvironmentContext, EnvironmentParser,
};
use crate::error::ParseError;

use super::matrix::{array_with_columns, matrix_column_count};

pub(crate) fn array_body(node: &ParseNode) -> Result<&[Vec<ParseNode>], ParseError> {
    match node {
        ParseNode::Array { body, .. } => Ok(body),
        _ => Err(ParseError::InternalInvariant {
            message: "Expected alignment array".to_string(),
        }),
    }
}

pub(crate) fn alignment_columns(count: usize, aligned: bool) -> Vec<ArrayColumn> {
    (0..count)
        .map(|index| {
            let alignment = if index % 2 == 0 { "r" } else { "l" };
            let pre_gap = if index % 2 == 0 && index > 0 && aligned {
                1.0
            } else {
                0.0
            };
            ArrayColumn::AlignColumn {
                alignment: alignment.to_string(),
                pre_gap: Some(pre_gap),
                post_gap: Some(0.0),
            }
        })
        .collect()
}

pub(crate) fn replace_alignment_columns(
    node: ParseNode,
    columns: Vec<ArrayColumn>,
) -> Result<ParseNode, ParseError> {
    array_with_columns(node, columns)
}

fn alignment_rhs_cell(cell: &ParseNode) -> Result<ParseNode, ParseError> {
    match cell {
        ParseNode::Styling {
            mode,
            style,
            reset_font,
            body,
        } => {
            let inner = match body.first() {
                Some(ParseNode::OrdGroup {
                    mode: inner_mode,
                    loc,
                    body: inner_body,
                    semisimple,
                }) => ParseNode::OrdGroup {
                    mode: *inner_mode,
                    loc: loc.clone(),
                    body: {
                        let mut new_body = vec![ParseNode::OrdGroup {
                            mode: *inner_mode,
                            loc: None,
                            body: Vec::new(),
                            semisimple: false,
                        }];
                        new_body.extend(inner_body.clone());
                        new_body
                    },
                    semisimple: *semisimple,
                },
                _ => {
                    return Err(ParseError::InternalInvariant {
                        message: "Expected alignment cell".to_string(),
                    });
                }
            };
            Ok(ParseNode::Styling {
                mode: *mode,
                style: *style,
                reset_font: *reset_font,
                body: vec![inner],
            })
        }
        _ => Err(ParseError::InternalInvariant {
            message: "Expected alignment cell".to_string(),
        }),
    }
}

pub(crate) fn insert_alignment_empty_groups(node: ParseNode) -> Result<ParseNode, ParseError> {
    match node {
        ParseNode::Array {
            mode,
            body,
            add_jot,
            array_stretch,
            columns,
            row_gaps,
            hskip_before_and_after,
            hlines_before_row,
            column_separation_type,
            tags,
            auto_tags,
            leqno,
        } => {
            let mut body = body;
            for row in &mut body {
                let mut index = 1;
                while index < row.len() {
                    let replacement = alignment_rhs_cell(&row[index])?;
                    row[index] = replacement;
                    index += 2;
                }
            }
            Ok(ParseNode::Array {
                mode,
                body,
                add_jot,
                array_stretch,
                columns,
                row_gaps,
                hskip_before_and_after,
                hlines_before_row,
                column_separation_type,
                tags,
                auto_tags,
                leqno,
            })
        }
        _ => Err(ParseError::InternalInvariant {
            message: "Expected alignment array".to_string(),
        }),
    }
}

fn alignment_display_mode(context: &EnvironmentContext) -> Result<(), ParseError> {
    if context.env_name != "aligned" && !context.display_mode {
        return Err(ParseError::InvalidArgument {
            message: format!("{{{}}} can be used only in display mode.", context.env_name),
            loc: None,
        });
    }
    Ok(())
}

pub(crate) fn aligned_environment_handler(
    parser: &mut dyn EnvironmentParser,
    context: &EnvironmentContext,
    _args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    alignment_display_mode(context)?;
    let is_split = context.env_name == "split";
    let auto_tag = match context.env_name.as_str() {
        "align" => Some(true),
        "align*" => Some(false),
        _ => None,
    };
    let array = parser.parse_array(ArrayEnvironmentOptions {
        columns: None,
        array_stretch: None,
        hskip_before_and_after: false,
        cell_style: StyleLevel::DisplayStyle,
        max_columns: if is_split { Some(2) } else { None },
        single_row: false,
        empty_single_row: true,
        auto_tag,
        leqno: context.leqno,
        add_jot: true,
        column_separation_type: Some(if context.env_name == "aligned" {
            ColumnSeparationType::AlignSeparation
        } else {
            ColumnSeparationType::AlignAtSeparation
        }),
    })?;
    let array = insert_alignment_empty_groups(array)?;
    let count = matrix_column_count(array_body(&array)?);
    replace_alignment_columns(
        array,
        alignment_columns(count, context.env_name == "aligned"),
    )
}

pub(crate) fn gather_environment_handler(
    parser: &mut dyn EnvironmentParser,
    context: &EnvironmentContext,
    _args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    if context.env_name != "gathered" && !context.display_mode {
        return Err(ParseError::InvalidArgument {
            message: format!("{{{}}} can be used only in display mode.", context.env_name),
            loc: None,
        });
    }
    let auto_tag = match context.env_name.as_str() {
        "gather" => Some(true),
        "gather*" => Some(false),
        _ => None,
    };
    parser.parse_array(ArrayEnvironmentOptions {
        columns: Some(vec![ArrayColumn::AlignColumn {
            alignment: "c".to_string(),
            pre_gap: None,
            post_gap: None,
        }]),
        array_stretch: None,
        hskip_before_and_after: false,
        cell_style: StyleLevel::DisplayStyle,
        max_columns: Some(1),
        single_row: false,
        empty_single_row: true,
        auto_tag,
        leqno: context.leqno,
        add_jot: true,
        column_separation_type: Some(ColumnSeparationType::GatherSeparation),
    })
}
