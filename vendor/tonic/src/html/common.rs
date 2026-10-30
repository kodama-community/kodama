use super::*;

pub(super) struct VElement {
    pub node: Node,
    pub shift: f64,
    pub margin_left: Option<String>,
    pub margin_right: Option<String>,
    pub classes: Vec<String>,
    pub style: Vec<(String, String)>,
}
pub(super) enum VChild {
    Elem(Box<VElement>),
    Kern(f64),
}
pub(super) fn elem(node: Node) -> VChild {
    shifted(node, 0.0)
}
pub(super) fn shifted(node: Node, shift: f64) -> VChild {
    VChild::Elem(Box::new(VElement {
        node,
        shift,
        margin_left: None,
        margin_right: None,
        classes: vec![],
        style: vec![],
    }))
}
impl VChild {
    pub fn margins(mut self, left: Option<String>, right: Option<String>) -> Self {
        if let Self::Elem(e) = &mut self {
            e.margin_left = left;
            e.margin_right = right;
        }
        self
    }
    pub fn classes(mut self, names: &[&str]) -> Self {
        if let Self::Elem(e) = &mut self {
            e.classes = classes(names);
        }
        self
    }
    pub fn style(mut self, key: &str, value: String) -> Self {
        if let Self::Elem(e) = &mut self {
            e.style.push((key.into(), value));
        }
        self
    }
}
pub(super) fn vlist(position: &str, data: f64, children: Vec<VChild>) -> Node {
    if children.is_empty() {
        return span(&["vlist-t"], vec![], None);
    }
    let (children, depth) = if position == "individualShift" {
        let mut source = children.into_iter();
        let VChild::Elem(first) = source.next().unwrap() else {
            return Node::error("Expected vlist element");
        };
        let depth = -first.shift - first.node.depth;
        let mut current = depth;
        let mut previous_size = first.node.height + first.node.depth;
        let mut output = vec![VChild::Elem(first)];
        for child in source {
            let VChild::Elem(item) = child else {
                return Node::error("Expected vlist element");
            };
            let diff = -item.shift - current - item.node.depth;
            let size = diff - previous_size;
            current += diff;
            previous_size = item.node.height + item.node.depth;
            output.push(VChild::Kern(size));
            output.push(VChild::Elem(item));
        }
        (output, depth)
    } else if position == "top" {
        let mut bottom = data;
        for child in &children {
            bottom -= match child {
                VChild::Kern(s) => *s,
                VChild::Elem(e) => e.node.height + e.node.depth,
            };
        }
        (children, bottom)
    } else if position == "bottom" {
        (children, -data)
    } else {
        let VChild::Elem(first) = &children[0] else {
            return Node::error("Expected vlist element");
        };
        let depth = -first.node.depth - if position == "shift" { data } else { 0.0 };
        (children, depth)
    };
    let psize = children.iter().fold(0.0_f64, |m, c| match c {
        VChild::Elem(e) => m.max(e.node.max_font_size).max(e.node.height),
        _ => m,
    }) + 2.0;
    let mut pstrut = span(&["pstrut"], vec![], None);
    pstrut.set_style("height", make_em(psize));
    let (mut min_pos, mut max_pos, mut current) = (depth, depth, depth);
    let mut real = vec![];
    for child in children {
        match child {
            VChild::Kern(s) => current += s,
            VChild::Elem(e) => {
                let size = e.node.height + e.node.depth;
                let top = -psize - current - e.node.depth;
                let mut wrapper = span_c(e.classes, vec![pstrut.clone(), e.node], None);
                for (k, v) in e.style {
                    wrapper.set_style(&k, v);
                }
                wrapper.set_style("top", make_em(top));
                if let Some(v) = e.margin_left {
                    wrapper.set_style("marginLeft", v);
                }
                if let Some(v) = e.margin_right {
                    wrapper.set_style("marginRight", v);
                }
                real.push(wrapper);
                current += size;
            }
        }
        min_pos = min_pos.min(current);
        max_pos = max_pos.max(current);
    }
    let mut list = span(&["vlist"], real, None);
    list.set_style("height", make_em(max_pos));
    let rows = if min_pos < 0.0 {
        let mut depth_strut = span(&["vlist"], vec![span(&[], vec![], None)], None);
        depth_strut.set_style("height", make_em(-min_pos));
        let top_strut = span(&["vlist-s"], vec![Node::text("symbol", "\u{200b}")], None);
        vec![
            span(&["vlist-r"], vec![list, top_strut], None),
            span(&["vlist-r"], vec![depth_strut], None),
        ]
    } else {
        vec![span(&["vlist-r"], vec![list], None)]
    };
    let mut table = span(
        if rows.len() == 2 {
            &["vlist-t", "vlist-t2"]
        } else {
            &["vlist-t"]
        },
        rows,
        None,
    );
    table.height = max_pos;
    table.depth = -min_pos;
    table
}
pub(super) fn line_span(class: &str, o: &Options, thickness: Option<f64>) -> Node {
    let mut line = span(&[class], vec![], Some(o));
    line.height = thickness
        .filter(|v| *v != 0.0)
        .unwrap_or(o.metric("defaultRuleThickness"))
        .max(o.min_rule_thickness);
    line.set_style("borderBottomWidth", make_em(line.height));
    line.max_font_size = 1.0;
    line
}
pub(super) fn combine_chars(chars: &mut Vec<Node>) {
    let mut i = 0;
    while i + 1 < chars.len() {
        let a = &chars[i];
        let b = &chars[i + 1];
        let compatible = a.kind == "symbol"
            && b.kind == "symbol"
            && class_string(&a.classes) == class_string(&b.classes)
            && a.skew == b.skew
            && a.max_font_size == b.max_font_size
            && !(a.italic != 0.0 && a.has_class("mathnormal"))
            && !(a.classes.len() == 1 && matches!(a.class(), "mbin" | "mord"))
            && a.style.len() == b.style.len()
            && a.style.iter().all(|v| b.style.contains(v));
        if compatible {
            let b = chars.remove(i + 1);
            let a = &mut chars[i];
            a.text.push_str(&b.text);
            a.height = a.height.max(b.height);
            a.depth = a.depth.max(b.depth);
            a.italic = b.italic;
        } else {
            i += 1;
        }
    }
}
