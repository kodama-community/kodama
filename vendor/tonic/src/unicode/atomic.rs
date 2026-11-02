use crate::ast::ParseNode;

/// True if the Unicode rendering has no internal operator that needs grouping.
///
/// A node is atomic when its rendered output is either a single token/character
/// or uses self-delimiting function-application notation.
pub(crate) fn is_atomic_expression(node: &ParseNode) -> bool {
    match node {
        ParseNode::Unsupported { fallback, .. } => is_atomic_expression(fallback),
        // --- Single-token leaf nodes ---
        ParseNode::Raw { .. }
        | ParseNode::ColorToken { .. }
        | ParseNode::TextOrd { .. }
        | ParseNode::MathOrd { .. }
        | ParseNode::Spacing { .. }
        | ParseNode::AccentToken { .. }
        | ParseNode::OperatorToken { .. }
        | ParseNode::Atom { .. }
        | ParseNode::Op { .. }
        | ParseNode::OperatorName { .. }
        | ParseNode::LeftRight { .. }
        | ParseNode::Font { .. }
        | ParseNode::DelimSizing { .. }
        | ParseNode::LeftRightRight { .. }
        | ParseNode::Middle { .. } => true,
        // --- Self-delimiting function notation (no external parens needed) ---
        ParseNode::Sqrt { .. }
        | ParseNode::Overline { .. }
        | ParseNode::Underline { .. }
        | ParseNode::Phantom { .. }
        | ParseNode::VPhantom { .. }
        | ParseNode::Pmb { .. }
        | ParseNode::Kern { .. }
        | ParseNode::Enclose { .. }
        | ParseNode::Rule { .. }
        | ParseNode::IncludeGraphics { .. }
        | ParseNode::AccentUnder { .. }
        | ParseNode::CdLabel { .. }
        | ParseNode::HorizBrace { .. }
        | ParseNode::XArrow { .. }
        | ParseNode::SupSub { .. } => true,
        // --- Body-array wrappers (atomic iff single child is atomic) ---
        ParseNode::Styling { body, .. }
        | ParseNode::Text { body, .. }
        | ParseNode::MClass { body, .. }
        | ParseNode::HBox { body, .. }
        | ParseNode::Sizing { body, .. }
        | ParseNode::Color { body, .. }
        | ParseNode::Href { body, .. }
        | ParseNode::Html { body, .. }
        | ParseNode::OrdGroup { body, .. } => body.len() == 1 && is_atomic_expression(&body[0]),
        // --- Transparent single-body wrappers (delegate to body) ---
        ParseNode::Accent { base, .. } => is_atomic_expression(base),
        ParseNode::Smash { body, .. }
        | ParseNode::VCenter { body, .. }
        | ParseNode::RaiseBox { body, .. }
        | ParseNode::ReflectBox { body, .. }
        | ParseNode::Lap { body, .. } => is_atomic_expression(body),
        ParseNode::CdParent { fragment, .. } => is_atomic_expression(fragment),
        _ => false,
    }
}
