use super::*;
fn atom_type(n: &Node) -> &str {
    if matches!(n.kind, "fragment" | "a") {
        if let Some(c) = n.children.last() {
            return atom_type(c);
        }
    }
    match n.class() {
        s @ ("mord" | "mop" | "mbin" | "mrel" | "mopen" | "mclose" | "mpunct" | "minner") => s,
        _ => "mord",
    }
}
fn script_element(n: Node, shift: f64, left: &str, right: &str) -> VChild {
    shifted(n, shift).margins((!left.is_empty()).then(|| left.into()), Some(right.into()))
}
pub(super) fn layout_supsub(
    base: Option<&ParseNode>,
    sup: Option<&ParseNode>,
    sub: Option<&ParseNode>,
    o: &Options,
) -> Node {
    use ParseNode::*;
    if let Some(node) = base {
        match node {
            Op {
                content,
                limits,
                always_handle_sup_sub,
                suppress_base_shift,
                ..
            } if *limits && (o.style / 2 == 0 || *always_handle_sup_sub) => {
                return layout_op(content, *suppress_base_shift, sup, sub, true, o)
            }
            OperatorName {
                body,
                always_handle_sup_sub,
                limits,
                ..
            } if *always_handle_sup_sub && (o.style / 2 == 0 || *limits) => {
                return layout_operator_name(body, sup, sub, true, o)
            }
            Accent {
                label,
                is_stretchy,
                is_shifty,
                base: inner,
                mode,
                ..
            } if character_box(inner) => {
                return layout_accent(
                    label,
                    *is_stretchy,
                    *is_shifty,
                    inner,
                    *mode,
                    sup,
                    sub,
                    true,
                    o,
                )
            }
            HorizBrace {
                label,
                is_over,
                base: inner,
                ..
            } if sub.is_none() == *is_over => {
                return layout_horiz_brace(label, *is_over, inner, sup, sub, o)
            }
            _ => (),
        }
    }
    let base_node = base.map_or_else(|| span(&[], vec![], None), |n| build_group(n, o));
    let is_char = base.is_some_and(character_box);
    let (mut sup_shift, mut sub_shift) = (0.0_f64, 0.0_f64);
    let sup_node = sup.map(|v| {
        let next = o.with_style(style_sup(o.style));
        let n = build_group_base(v, &next, o);
        if !is_char {
            sup_shift = base_node.height
                - next.metric("supDrop") * next.size_multiplier / o.size_multiplier;
        }
        n
    });
    let sub_node = sub.map(|v| {
        let next = o.with_style(style_sub(o.style));
        let n = build_group_base(v, &next, o);
        if !is_char {
            sub_shift =
                base_node.depth + next.metric("subDrop") * next.size_multiplier / o.size_multiplier;
        }
        n
    });
    let min_sup = o.metric(if o.style == 0 {
        "sup1"
    } else if o.style % 2 == 1 {
        "sup3"
    } else {
        "sup2"
    });
    let right = make_em(0.5 / o.metric("ptPerEm") / o.size_multiplier);
    let oval = matches!(base,Some(Op{content:OperatorContent::SymbolOperator(s),..})if matches!(s.as_str(),"\\oiint"|"\\oiiint"));
    let left = if base_node.kind == "symbol" || oval {
        make_em(-base_node.italic)
    } else {
        String::new()
    };
    let x = o.metric("xHeight");
    let scripts = match (sup_node, sub_node) {
        (Some(upper), Some(lower)) => {
            sup_shift = sup_shift.max(min_sup).max(upper.depth + 0.25 * x);
            sub_shift = sub_shift.max(o.metric("sub2"));
            let width = 4.0 * o.metric("defaultRuleThickness");
            if sup_shift - upper.depth - (lower.height - sub_shift) < width {
                sub_shift = width - (sup_shift - upper.depth) + lower.height;
                let psi = 0.8 * x - (sup_shift - upper.depth);
                if psi > 0.0 {
                    sup_shift += psi;
                    sub_shift -= psi;
                }
            }
            vlist(
                "individualShift",
                0.0,
                vec![
                    script_element(lower, sub_shift, &left, &right),
                    shifted(upper, -sup_shift).margins(None, Some(right)),
                ],
            )
        }
        (None, Some(lower)) => {
            sub_shift = sub_shift.max(o.metric("sub1")).max(lower.height - 0.8 * x);
            vlist(
                "shift",
                sub_shift,
                vec![script_element(lower, 0.0, &left, &right)],
            )
        }
        (Some(upper), None) => {
            sup_shift = sup_shift.max(min_sup).max(upper.depth + 0.25 * x);
            vlist(
                "shift",
                -sup_shift,
                vec![elem(upper).margins(None, Some(right))],
            )
        }
        (None, None) => return base_node,
    };
    let class = atom_type(&base_node).to_string();
    span_c(
        vec![class],
        vec![base_node, span(&["msupsub"], vec![scripts], None)],
        Some(o),
    )
}
pub(super) fn layout_op(
    content: &OperatorContent,
    suppress_shift: bool,
    sup: Option<&ParseNode>,
    sub: Option<&ParseNode>,
    limits: bool,
    o: &Options,
) -> Node {
    let mut base = match content {
        OperatorContent::SymbolOperator(name) => {
            let large = o.style / 2 == 0 && name != "\\smallint";
            let font = if large {
                "Size2-Regular"
            } else {
                "Size1-Regular"
            };
            let glyph = match name.as_str() {
                "\\oiint" => "\\iint",
                "\\oiiint" => "\\iiint",
                _ => name,
            };
            let symbol = make_symbol(
                glyph,
                font,
                Mode::Math,
                o,
                classes(&[
                    "mop",
                    "op-symbol",
                    if large { "large-op" } else { "small-op" },
                ]),
            );
            if matches!(name.as_str(), "\\oiint" | "\\oiiint") {
                let oval = static_svg(
                    &format!(
                        "{}{}",
                        if name == "\\oiint" {
                            "oiintSize"
                        } else {
                            "oiiintSize"
                        },
                        if large { 2 } else { 1 }
                    ),
                    o,
                );
                let italic = symbol.italic;
                let mut wrapped = vlist(
                    "individualShift",
                    0.0,
                    vec![elem(symbol), shifted(oval, if large { 0.08 } else { 0.0 })],
                );
                wrapped.classes.insert(0, "mop".into());
                wrapped.italic = italic;
                wrapped
            } else {
                symbol
            }
        }
        OperatorContent::BodyOperator(body) => {
            let mut inner = build_expression(body, o);
            if inner.len() == 1 && inner[0].kind == "symbol" {
                let mut n = inner.remove(0);
                n.classes[0] = "mop".into();
                n
            } else {
                span(&["mop"], inner, Some(o))
            }
        }
        OperatorContent::NamedOperator(name) => {
            let name = name.strip_prefix('\\').unwrap_or(name);
            span(
                &["mop"],
                name.chars()
                    .map(|c| math_symbol(&c.to_string(), Mode::Math, o, vec![]))
                    .collect(),
                Some(o),
            )
        }
    };
    let oval = matches!(content,OperatorContent::SymbolOperator(s)if matches!(s.as_str(),"\\oiint"|"\\oiiint"));
    let shifted = (base.kind == "symbol" || oval) && !suppress_shift;
    let shift = if shifted {
        (base.height - base.depth) / 2.0 - o.metric("axisHeight")
    } else {
        0.0
    };
    let slant = if shifted { base.italic } else { 0.0 };
    if limits {
        layout_limits(base, sup, sub, slant, shift, o)
    } else {
        if shift != 0.0 {
            base.set_style("position", "relative");
            base.set_style("top", make_em(shift));
        }
        base
    }
}
pub(super) fn layout_operator_name(
    body: &[ParseNode],
    sup: Option<&ParseNode>,
    sub: Option<&ParseNode>,
    limits: bool,
    o: &Options,
) -> Node {
    let converted = body
        .iter()
        .map(|n| match n {
            ParseNode::MathOrd { text, mode, .. }
            | ParseNode::TextOrd { text, mode, .. }
            | ParseNode::Atom { text, mode, .. }
            | ParseNode::Spacing { text, mode, .. }
            | ParseNode::AccentToken { text, mode, .. }
            | ParseNode::OperatorToken { text, mode, .. } => ParseNode::TextOrd {
                mode: *mode,
                text: text.clone(),
                loc: None,
            },
            _ => n.clone(),
        })
        .collect::<Vec<_>>();
    let mut inner = build_expression(
        &converted,
        &Options {
            font: "mathrm".into(),
            ..o.clone()
        },
    );
    for n in &mut inner {
        if n.kind == "symbol" {
            n.text = n.text.replace('−', "-").replace('∗', "*");
        }
    }
    let base = span(&["mop"], inner, Some(o));
    if limits {
        layout_limits(base, sup, sub, 0.0, 0.0, o)
    } else {
        base
    }
}
fn layout_limits(
    base: Node,
    sup: Option<&ParseNode>,
    sub: Option<&ParseNode>,
    slant: f64,
    base_shift: f64,
    o: &Options,
) -> Node {
    let base = span(&[], vec![base], None);
    let sub_single = sub.is_some_and(character_box);
    let upper = sup.map(|n| {
        let e = build_group_base(n, &o.with_style(style_sup(o.style)), o);
        let k = o
            .metric("bigOpSpacing1")
            .max(o.metric("bigOpSpacing3") - e.depth);
        (e, k)
    });
    let lower = sub.map(|n| {
        let e = build_group_base(n, &o.with_style(style_sub(o.style)), o);
        let k = o
            .metric("bigOpSpacing2")
            .max(o.metric("bigOpSpacing4") - e.height);
        (e, k)
    });
    let has_lower = lower.is_some();
    let spacing = o.metric("bigOpSpacing5");
    let group = match (upper, lower) {
        (Some((u, uk)), Some((l, lk))) => {
            let bottom = spacing + l.height + l.depth + lk + base.depth + base_shift;
            vlist(
                "bottom",
                bottom,
                vec![
                    VChild::Kern(spacing),
                    elem(l).margins(Some(make_em(-slant)), None),
                    VChild::Kern(lk),
                    elem(base),
                    VChild::Kern(uk),
                    elem(u).margins(Some(make_em(slant)), None),
                    VChild::Kern(spacing),
                ],
            )
        }
        (None, Some((l, k))) => vlist(
            "top",
            base.height - base_shift,
            vec![
                VChild::Kern(spacing),
                elem(l).margins(Some(make_em(-slant)), None),
                VChild::Kern(k),
                elem(base),
            ],
        ),
        (Some((u, k)), None) => vlist(
            "bottom",
            base.depth + base_shift,
            vec![
                elem(base),
                VChild::Kern(k),
                elem(u).margins(Some(make_em(slant)), None),
                VChild::Kern(spacing),
            ],
        ),
        (None, None) => return base,
    };
    let mut parts = vec![];
    if has_lower && slant != 0.0 && !sub_single {
        let mut s = span(&["mspace"], vec![], Some(o));
        s.set_style("marginRight", make_em(slant));
        parts.push(s);
    }
    parts.push(group);
    span(&["mop", "op-limits"], parts, Some(o))
}
