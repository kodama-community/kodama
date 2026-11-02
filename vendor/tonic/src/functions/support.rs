use crate::ast::ParseNode;
use crate::error::ParseError;

pub(crate) fn require_function_arg(
    args: &[ParseNode],
    index: usize,
    func_name: &str,
) -> Result<ParseNode, ParseError> {
    args.get(index)
        .cloned()
        .ok_or_else(|| ParseError::InternalInvariant {
            message: format!("Missing argument {index} for {func_name}"),
        })
}

pub(crate) fn ord_argument(arg: ParseNode) -> Vec<ParseNode> {
    if let ParseNode::OrdGroup { body, .. } = arg {
        body
    } else {
        vec![arg]
    }
}

pub(crate) fn normalize_argument(arg: ParseNode) -> ParseNode {
    if let ParseNode::OrdGroup { body, .. } = &arg {
        if body.len() == 1 {
            return body[0].clone();
        }
    }
    arg
}
