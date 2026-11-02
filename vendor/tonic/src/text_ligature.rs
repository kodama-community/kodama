use crate::ast::{Mode, ParseNode};
use crate::source_location::SourceLocation;

fn merged_text_location(first: &ParseNode, last: &ParseNode) -> Option<SourceLocation> {
    match (first, last) {
        (
            ParseNode::TextOrd {
                loc: Some(start), ..
            },
            ParseNode::TextOrd { loc: Some(end), .. },
        ) if start.input == end.input => Some(SourceLocation::range(start, end)),
        _ => None,
    }
}

fn merged_text_ord(first: &ParseNode, last: &ParseNode, text: String) -> ParseNode {
    ParseNode::TextOrd {
        mode: Mode::Text,
        loc: merged_text_location(first, last),
        text,
    }
}

fn is_textord(node: &ParseNode, text: &str) -> bool {
    matches!(node, ParseNode::TextOrd { text: t, .. } if t.as_str() == text)
}

fn ligature_run(nodes: &[ParseNode]) -> Option<(usize, String)> {
    let ParseNode::TextOrd { text, .. } = nodes.first()? else {
        return None;
    };
    let text = text.as_str();
    let cap = match text {
        "-" => 3,
        "'" | "`" => 2,
        _ => return None,
    };
    let run = nodes.iter().take_while(|n| is_textord(n, text)).count();
    let take = run.min(cap);
    (take >= 2).then(|| (take, text.repeat(take)))
}

fn next_node(nodes: &[ParseNode]) -> (usize, ParseNode) {
    let Some((take, text)) = ligature_run(nodes) else {
        return (1, nodes[0].clone());
    };
    (take, merged_text_ord(&nodes[0], &nodes[take - 1], text))
}

pub(crate) fn form_text_ligatures(body: Vec<ParseNode>) -> Vec<ParseNode> {
    let mut output = Vec::with_capacity(body.len());
    let mut rest: &[ParseNode] = &body;
    while !rest.is_empty() {
        let (take, node) = next_node(rest);
        output.push(node);
        rest = &rest[take..];
    }
    output
}
