use super::*;

pub(super) fn build_expression(nodes: &[ParseNode], o: &Options) -> Vec<Node> {
    expression(nodes, o, true, false, "leftmost", "rightmost")
}
pub(super) fn partial_expression(nodes: &[ParseNode], o: &Options) -> Vec<Node> {
    expression(nodes, o, false, false, "leftmost", "rightmost")
}

pub(super) fn expression(
    nodes: &[ParseNode],
    o: &Options,
    real: bool,
    root: bool,
    left: &str,
    right: &str,
) -> Vec<Node> {
    let mut groups = vec![];
    for n in nodes {
        let output = build_group(n, o);
        if output.kind == "fragment" {
            groups.extend(output.children);
        } else {
            groups.push(output);
        }
    }
    combine_chars(&mut groups);
    if !real {
        return groups;
    }
    let glue_options = match nodes {
        [ParseNode::Sizing { size, .. }] if (1..=11).contains(size) => o.with_size(*size),
        [ParseNode::Styling { style, .. }] => o.with_style(style_id(*style)),
        _ => o.clone(),
    };
    let mut visits = vec![];
    collect(&groups, &mut vec![], &mut visits);
    let mut prev_class = left.to_string();
    let mut previous: Option<&Vec<usize>> = None;
    for path in &visits {
        let n = at(&groups, path);
        if !n.has_class("mspace") {
            let current = n.class().to_string();
            if prev_class == "mbin"
                && matches!(current.as_str(), "rightmost" | "mrel" | "mclose" | "mpunct")
            {
                if let Some(p) = previous {
                    at_mut(&mut groups, p).classes[0] = "mord".into();
                }
            } else if current == "mbin"
                && matches!(
                    prev_class.as_str(),
                    "leftmost" | "mbin" | "mopen" | "mrel" | "mop" | "mpunct"
                )
            {
                at_mut(&mut groups, path).classes[0] = "mord".into();
            }
            prev_class = at(&groups, path).class().into();
            previous = Some(path);
        } else if root && n.has_class("katex-newline") {
            prev_class = "leftmost".into();
            previous = None;
        }
    }
    if prev_class == "mbin" && matches!(right, "rightmost" | "mrel" | "mclose" | "mpunct") {
        if let Some(p) = previous {
            at_mut(&mut groups, p).classes[0] = "mord".into();
        }
    }
    let mut insertions = vec![];
    let mut insert_parent = vec![];
    let mut insert_index = 0;
    prev_class = left.into();
    for p in &visits {
        let n = at(&groups, p);
        if !n.has_class("mspace") {
            if let Some(glue) =
                implicit_glue(&prev_class, n.class(), n.has_class("mtight"), &glue_options)
            {
                insertions.push((insert_parent.clone(), insert_index, glue));
            }
            prev_class = n.class().into();
        } else if root && n.has_class("katex-newline") {
            prev_class = "leftmost".into();
        }
        insert_parent = p[..p.len() - 1].to_vec();
        insert_index = p[p.len() - 1] + 1;
    }
    if let Some(glue) = implicit_glue(&prev_class, right, o.style >= 4, &glue_options) {
        insertions.push((insert_parent, insert_index, glue));
    }
    for (p, i, g) in insertions.into_iter().rev() {
        if p.is_empty() {
            groups.insert(i, g);
        } else {
            at_mut(&mut groups, &p).children.insert(i, g);
        }
    }
    groups
}
fn collect(nodes: &[Node], path: &mut Vec<usize>, out: &mut Vec<Vec<usize>>) {
    for (i, n) in nodes.iter().enumerate() {
        path.push(i);
        if n.kind == "fragment" || n.kind == "a" || (n.kind == "span" && n.has_class("enclosing")) {
            collect(&n.children, path, out);
        } else {
            out.push(path.clone());
        }
        path.pop();
    }
}
fn at<'a>(nodes: &'a [Node], path: &[usize]) -> &'a Node {
    let n = &nodes[path[0]];
    if path.len() == 1 {
        n
    } else {
        at(&n.children, &path[1..])
    }
}
fn at_mut<'a>(nodes: &'a mut [Node], path: &[usize]) -> &'a mut Node {
    let n = &mut nodes[path[0]];
    if path.len() == 1 {
        n
    } else {
        at_mut(&mut n.children, &path[1..])
    }
}
fn implicit_glue(left: &str, right: &str, tight: bool, o: &Options) -> Option<Node> {
    let spec = crate::SpacingSpec {
        thin: "3".into(),
        operator: "3".into(),
        medium: "4".into(),
        thick: "5".into(),
    };
    crate::math_spacing(left, right, tight, &spec).map(|n| {
        glue(
            &Measurement {
                number: match n.as_str() {
                    "3" => 3.0,
                    "4" => 4.0,
                    _ => 5.0,
                },
                unit: "mu".into(),
            },
            o,
        )
    })
}
