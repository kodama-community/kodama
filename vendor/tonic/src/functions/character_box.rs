use crate::ast::{AtomFamily, ParseNode};

pub(crate) fn base_element(node: &ParseNode) -> &ParseNode {
    match node {
        ParseNode::OrdGroup { body, .. } => {
            if body.len() == 1 {
                base_element(&body[0])
            } else {
                node
            }
        }
        ParseNode::Color { body, .. } => {
            if body.len() == 1 {
                base_element(&body[0])
            } else {
                node
            }
        }
        ParseNode::Font { body, .. } => base_element(body),
        _ => node,
    }
}

pub(crate) fn is_character_box(node: &ParseNode) -> bool {
    matches!(
        base_element(node),
        ParseNode::MathOrd { .. } | ParseNode::TextOrd { .. } | ParseNode::Atom { .. }
    )
}

pub(crate) fn binrel_class(arg: &ParseNode) -> AtomFamily {
    let atom = match arg {
        ParseNode::OrdGroup { body, .. } => match body.first() {
            Some(first) => first,
            None => arg,
        },
        _ => arg,
    };
    match atom {
        ParseNode::Atom {
            family: AtomFamily::Mbin,
            ..
        } => AtomFamily::Mbin,
        ParseNode::Atom {
            family: AtomFamily::Mrel,
            ..
        } => AtomFamily::Mrel,
        _ => AtomFamily::Mord,
    }
}
