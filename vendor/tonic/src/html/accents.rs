use super::*;
fn base_skew(n: &Node) -> f64 {
    if n.kind == "symbol" {
        n.skew
    } else if n.children.len() == 1 {
        base_skew(&n.children[0])
    } else {
        0.0
    }
}
#[allow(clippy::too_many_arguments)]
pub(super) fn layout_accent(
    label: &str,
    stretchy: bool,
    shifty: bool,
    base: &ParseNode,
    mode: Mode,
    sup: Option<&ParseNode>,
    sub: Option<&ParseNode>,
    has_scripts: bool,
    o: &Options,
) -> Node {
    let scripts = has_scripts.then(|| layout_supsub(Some(base), sup, sub, o));
    let body = build_group(base, &o.with_style(style_cramp(o.style)));
    let skew = if shifty && character_box(base) {
        base_skew(&body)
    } else {
        0.0
    };
    let below = label == "\\c";
    let mut clearance = if below {
        body.height + body.depth
    } else {
        body.height.min(o.metric("xHeight"))
    };
    let accent_body = if !stretchy {
        let (accent, width) = if label == "\\vec" {
            (static_svg("vec", o), 0.471)
        } else {
            let mut n = make_ord(
                &ParseNode::TextOrd {
                    mode,
                    text: label.into(),
                    loc: None,
                },
                o,
            );
            n.italic = 0.0;
            if below {
                clearance += n.depth;
            }
            let w = n.width;
            (n, w)
        };
        let mut accent_body = span(&["accent-body"], vec![accent], None);
        let full = label == "\\textcircled";
        if full {
            accent_body.classes.push("accent-full".into());
            clearance = body.height;
        }
        accent_body.set_style(
            "left",
            make_em(if full { skew } else { skew - width / 2.0 }),
        );
        if full {
            accent_body.set_style("top", ".2em");
        }
        vlist(
            "firstBaseline",
            0.0,
            vec![elem(body), VChild::Kern(-clearance), elem(accent_body)],
        )
    } else {
        let mut accent = elem(stretchy_svg(label, Some(base), o)).classes(&["svg-align"]);
        if skew > 0.0 {
            accent = accent
                .style("width", format!("calc(100% - {})", make_em(2.0 * skew)))
                .style("marginLeft", make_em(2.0 * skew));
        }
        vlist("firstBaseline", 0.0, vec![elem(body), accent])
    };
    let result = span(&["mord", "katex-accent"], vec![accent_body], Some(o));
    if let Some(mut scripts) = scripts {
        scripts.height = scripts.height.max(result.height);
        scripts.children[0] = result;
        scripts.classes[0] = "mord".into();
        scripts
    } else {
        result
    }
}
pub(super) fn layout_accent_under(label: &str, base: &ParseNode, o: &Options) -> Node {
    let inner = build_group(base, o);
    let accent = stretchy_svg(label, Some(base), o);
    let kern = if label == "\\utilde" { 0.12 } else { 0.0 };
    let list = vlist(
        "top",
        inner.height,
        vec![
            elem(accent).classes(&["svg-align"]),
            VChild::Kern(kern),
            elem(inner),
        ],
    );
    span(&["mord", "accentunder"], vec![list], Some(o))
}
pub(super) fn layout_horiz_brace(
    label: &str,
    over: bool,
    base: &ParseNode,
    sup: Option<&ParseNode>,
    sub: Option<&ParseNode>,
    o: &Options,
) -> Node {
    let scripts = if let Some(n) = sup {
        Some(build_group_base(n, &o.with_style(style_sup(o.style)), o))
    } else {
        sub.map(|n| build_group_base(n, &o.with_style(style_sub(o.style)), o))
    };
    let body = build_group(base, &o.base_style(0));
    let brace = stretchy_svg(label, Some(base), o);
    let mut list = if over {
        vlist(
            "firstBaseline",
            0.0,
            vec![
                elem(body),
                VChild::Kern(0.1),
                elem(brace).classes(&["svg-align"]),
            ],
        )
    } else {
        vlist(
            "bottom",
            body.depth + 0.1 + brace.height,
            vec![
                elem(brace).classes(&["svg-align"]),
                VChild::Kern(0.1),
                elem(body),
            ],
        )
    };
    if let Some(scripts) = scripts {
        let wrapped = span(
            &["minner", if over { "mover" } else { "munder" }],
            vec![list],
            Some(o),
        );
        list = if over {
            vlist(
                "firstBaseline",
                0.0,
                vec![elem(wrapped), VChild::Kern(0.2), elem(scripts)],
            )
        } else {
            vlist(
                "bottom",
                wrapped.depth + 0.2 + scripts.height + scripts.depth,
                vec![elem(scripts), VChild::Kern(0.2), elem(wrapped)],
            )
        };
    }
    span(
        &["minner", if over { "mover" } else { "munder" }],
        vec![list],
        Some(o),
    )
}
pub(super) fn layout_xarrow(
    label: &str,
    body: &ParseNode,
    below: Option<&ParseNode>,
    o: &Options,
) -> Node {
    let mut upper = if label == "\\\\cdlongequal"
        && matches!(body,ParseNode::OrdGroup{body,..}if body.is_empty())
    {
        span(&[], vec![], None)
    } else {
        wrap_fragment(
            build_group_base(body, &o.with_style(style_sup(o.style)), o),
            o,
        )
    };
    let prefix = if label.starts_with("\\x") { "x" } else { "cd" };
    upper.classes.push(format!("{prefix}-arrow-pad"));
    let lower = below.map(|n| {
        let mut b = wrap_fragment(build_group_base(n, &o.with_style(style_sub(o.style)), o), o);
        b.classes.push(format!("{prefix}-arrow-pad"));
        b
    });
    let arrow = stretchy_svg(label, None, o);
    let axis = o.metric("axisHeight");
    let ah = arrow.height;
    let arrow_shift = -axis + 0.5 * ah;
    let upper_shift = -axis
        - 0.5 * ah
        - 0.111
        - if upper.depth > 0.25 || label == "\\xleftequilibrium" {
            upper.depth
        } else {
            0.0
        };
    let mut children = vec![
        shifted(upper, upper_shift),
        shifted(arrow, arrow_shift).classes(&["svg-align"]),
    ];
    if let Some(lower) = lower {
        let shift = -axis + lower.height + 0.5 * ah + 0.111;
        children.push(shifted(lower, shift));
    }
    span(
        &["mrel", "x-arrow"],
        vec![vlist("individualShift", 0.0, children)],
        Some(o),
    )
}
