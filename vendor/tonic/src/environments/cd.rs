use crate::ast::{AtomFamily, Mode, ParseNode, StyleLevel};
use crate::environments::registry::{EnvironmentContext, EnvironmentParser};
use crate::error::ParseError;

fn cd_cell(body: Vec<ParseNode>) -> ParseNode {
    ParseNode::Styling {
        mode: Mode::Math,
        body,
        style: StyleLevel::DisplayStyle,
        reset_font: true,
    }
}

fn cd_arrow_character(node: &ParseNode) -> Option<String> {
    match node {
        ParseNode::MathOrd { text, .. }
        | ParseNode::TextOrd { text, .. }
        | ParseNode::Atom { text, .. } => Some(text.clone()),
        _ => None,
    }
}

fn cd_label_end(node: &ParseNode, end: &str) -> bool {
    match node {
        ParseNode::MathOrd { text, .. } | ParseNode::Atom { text, .. } => text == end,
        _ => false,
    }
}

fn cd_empty_label() -> ParseNode {
    ParseNode::OrdGroup {
        mode: Mode::Math,
        loc: None,
        body: Vec::new(),
        semisimple: false,
    }
}

fn cd_arrow(arrow: &str, upper: ParseNode, lower: ParseNode) -> Result<ParseNode, ParseError> {
    let node = match arrow {
        ">" => ParseNode::XArrow {
            mode: Mode::Math,
            label: "\\\\cdrightarrow".to_string(),
            body: Box::new(upper),
            below: Some(Box::new(lower)),
        },
        "<" => ParseNode::XArrow {
            mode: Mode::Math,
            label: "\\\\cdleftarrow".to_string(),
            body: Box::new(upper),
            below: Some(Box::new(lower)),
        },
        "=" => ParseNode::XArrow {
            mode: Mode::Math,
            label: "\\\\cdlongequal".to_string(),
            body: Box::new(cd_empty_label()),
            below: None,
        },
        "|" => ParseNode::DelimSizing {
            mode: Mode::Math,
            size: 2,
            mclass: AtomFamily::Mord,
            delim: "\\Vert".to_string(),
        },
        "." => ParseNode::TextOrd {
            mode: Mode::Math,
            loc: None,
            text: " ".to_string(),
        },
        "A" | "V" => {
            let direction = if arrow == "A" {
                "\\uparrow"
            } else {
                "\\downarrow"
            };
            ParseNode::CdParent {
                mode: Mode::Math,
                fragment: Box::new(ParseNode::OrdGroup {
                    mode: Mode::Math,
                    loc: None,
                    body: vec![
                        ParseNode::CdLabel {
                            mode: Mode::Math,
                            side: "left".to_string(),
                            label: Box::new(upper),
                        },
                        ParseNode::DelimSizing {
                            mode: Mode::Math,
                            size: 2,
                            mclass: AtomFamily::Mord,
                            delim: direction.to_string(),
                        },
                        ParseNode::CdLabel {
                            mode: Mode::Math,
                            side: "right".to_string(),
                            label: Box::new(lower),
                        },
                    ],
                    semisimple: false,
                }),
            }
        }
        _ => {
            return Err(ParseError::InvalidArgument {
                message: "Expected one of \"<>AV=|.\" after @".to_string(),
                loc: None,
            });
        }
    };
    Ok(node)
}

fn scan_cd_label(
    nodes: &[ParseNode],
    mut index: usize,
    arrow: &str,
) -> Result<(Vec<ParseNode>, usize), ParseError> {
    let mut label: Vec<ParseNode> = Vec::new();
    while index < nodes.len() {
        let current = &nodes[index];
        if cd_label_end(current, arrow) {
            return Ok((label, index + 1));
        }
        if let ParseNode::TextOrd { text, .. } = current {
            if text == "@" {
                return Err(ParseError::InvalidArgument {
                    message: format!("Missing a {arrow} character to complete a CD arrow."),
                    loc: None,
                });
            }
        }
        label.push(current.clone());
        index += 1;
    }
    Err(ParseError::InvalidArgument {
        message: format!("Missing a {arrow} character to complete a CD arrow."),
        loc: None,
    })
}

pub(crate) fn cd_row(nodes: Vec<ParseNode>, even: bool) -> Result<Vec<ParseNode>, ParseError> {
    let mut row: Vec<ParseNode> = Vec::new();
    let mut cell: Vec<ParseNode> = Vec::new();
    let mut index = 0;
    while index < nodes.len() {
        let node = &nodes[index];
        if let ParseNode::TextOrd { text, .. } = node {
            if text == "@" {
                row.push(cd_cell(cell.clone()));
                cell.clear();
                index += 1;
                let Some(character_node) = nodes.get(index) else {
                    return Err(ParseError::InvalidArgument {
                        message: "Expected one of \"<>AV=|.\" after @".to_string(),
                        loc: None,
                    });
                };
                let Some(arrow) = cd_arrow_character(character_node) else {
                    return Err(ParseError::InvalidArgument {
                        message: "Expected one of \"<>AV=|.\" after @".to_string(),
                        loc: None,
                    });
                };
                index += 1;
                let mut labels: Vec<ParseNode> = Vec::new();
                if arrow == ">" || arrow == "<" || arrow == "A" || arrow == "V" {
                    for _ in 0..2 {
                        let (label_body, next_index) = scan_cd_label(&nodes, index, &arrow)?;
                        index = next_index;
                        labels.push(ParseNode::OrdGroup {
                            mode: Mode::Math,
                            loc: None,
                            body: label_body,
                            semisimple: false,
                        });
                    }
                } else if arrow != "=" && arrow != "|" && arrow != "." {
                    return Err(ParseError::InvalidArgument {
                        message: "Expected one of \"<>AV=|.\" after @".to_string(),
                        loc: None,
                    });
                } else {
                    labels.push(cd_empty_label());
                    labels.push(cd_empty_label());
                }
                row.push(cd_cell(vec![cd_arrow(
                    &arrow,
                    labels[0].clone(),
                    labels[1].clone(),
                )?]));
                continue;
            }
        }
        cell.push(node.clone());
        index += 1;
    }
    if even {
        row.push(cd_cell(cell));
    } else if !row.is_empty() {
        row.remove(0);
    }
    Ok(row)
}

pub(crate) fn cd_environment_handler(
    parser: &mut dyn EnvironmentParser,
    context: &EnvironmentContext,
    _args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    if !context.display_mode {
        return Err(ParseError::InvalidArgument {
            message: "{CD} can be used only in display mode.".to_string(),
            loc: None,
        });
    }
    parser.parse_cd()
}
