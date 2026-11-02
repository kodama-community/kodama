use super::*;
pub(super) fn wrap_fragment(node: Node, o: &Options) -> Node {
    if node.kind == "fragment" {
        span(&[], vec![node], Some(o))
    } else {
        node
    }
}
pub(super) fn character_box(node: &ParseNode) -> bool {
    crate::functions::is_character_box(node)
}
pub(super) fn layout_node(node: &ParseNode, o: &Options) -> Option<Node> {
    use ParseNode::*;
    Some(match node {
        Sqrt { body, index, .. } => layout_sqrt(body, index.as_deref(), o),
        GenFrac {
            numer,
            denom,
            continued,
            has_bar_line,
            bar_size,
            left_delim,
            right_delim,
            mode,
        } => layout_fraction(
            numer,
            denom,
            *continued,
            *has_bar_line,
            bar_size.as_ref(),
            left_delim.as_deref(),
            right_delim.as_deref(),
            *mode,
            o,
        ),
        SupSub { base, sup, sub, .. } => {
            layout_supsub(base.as_deref(), sup.as_deref(), sub.as_deref(), o)
        }
        Op {
            content,
            suppress_base_shift,
            ..
        } => layout_op(content, *suppress_base_shift, None, None, false, o),
        OperatorName { body, .. } => layout_operator_name(body, None, None, false, o),
        Accent {
            label,
            is_stretchy,
            is_shifty,
            base,
            mode,
            ..
        } => layout_accent(
            label,
            *is_stretchy,
            *is_shifty,
            base,
            *mode,
            None,
            None,
            false,
            o,
        ),
        AccentUnder { label, base, .. } => layout_accent_under(label, base, o),
        HorizBrace {
            label,
            is_over,
            base,
            ..
        } => layout_horiz_brace(label, *is_over, base, None, None, o),
        XArrow {
            label, body, below, ..
        } => layout_xarrow(label, body, below.as_deref(), o),
        Overline { body, .. } => layout_line(body, true, o),
        Underline { body, .. } => layout_line(body, false, o),
        DelimSizing {
            delim,
            size,
            mclass,
            mode,
        } => {
            if delim == "." {
                span_c(vec![atom_class(*mclass)], vec![], None)
            } else {
                sized_delim(delim, *size, o, *mode, vec![atom_class(*mclass)])
            }
        }
        Middle { delim, mode } => {
            let mut result = if delim == "." {
                null_delimiter(o, vec![])
            } else {
                sized_delim(delim, 1, o, *mode, vec![])
            };
            if delim != "." {
                result.middle = Some((delim.clone(), o.clone()));
            }
            result
        }
        LeftRight {
            body,
            left,
            right,
            right_color,
            mode,
        } => layout_left_right(body, left, right, right_color.as_deref(), *mode, o),
        LeftRightRight { delim, color, mode } => {
            if delim == "." {
                null_delimiter(o, classes(&["mclose"]))
            } else {
                sized_delim(
                    delim,
                    1,
                    &Options {
                        color: color.clone().or_else(|| o.color.clone()),
                        ..o.clone()
                    },
                    *mode,
                    classes(&["mclose"]),
                )
            }
        }
        _ => return None,
    })
}
fn layout_left_right(
    body: &[ParseNode],
    left: &str,
    right: &str,
    color: Option<&str>,
    mode: Mode,
    o: &Options,
) -> Node {
    let mut inner = expression(body, o, true, false, "mopen", "mclose");
    let (mut height, mut depth) = (0.0_f64, 0.0_f64);
    for n in &inner {
        if n.middle.is_none() {
            height = height.max(n.height);
            depth = depth.max(n.depth);
        }
    }
    height *= o.size_multiplier;
    depth *= o.size_multiplier;
    let left = if left == "." {
        null_delimiter(o, classes(&["mopen"]))
    } else {
        left_right_delim(left, height, depth, o, mode, classes(&["mopen"]))
    };
    for n in &mut inner {
        if let Some((delim, options)) = &n.middle {
            *n = left_right_delim(delim, height, depth, options, mode, vec![]);
        }
    }
    let right_options = Options {
        color: color.map(str::to_string).or_else(|| o.color.clone()),
        ..o.clone()
    };
    let right = if right == "." {
        null_delimiter(o, classes(&["mclose"]))
    } else {
        left_right_delim(
            right,
            height,
            depth,
            &right_options,
            mode,
            classes(&["mclose"]),
        )
    };
    inner.insert(0, left);
    inner.push(right);
    span(&["minner"], inner, Some(o))
}
fn layout_sqrt(body: &ParseNode, index: Option<&ParseNode>, o: &Options) -> Node {
    let mut inner = wrap_fragment(build_group(body, &o.with_style(style_cramp(o.style))), o);
    if inner.height == 0.0 {
        inner.height = o.metric("xHeight");
    }
    let theta = o.metric("defaultRuleThickness");
    let phi = if o.style < 2 {
        o.metric("xHeight")
    } else {
        theta
    };
    let mut clearance = theta + phi / 4.0;
    let (image, rule_width, advance) =
        sqrt_image(inner.height + inner.depth + clearance + theta, o);
    let delim_depth = image.height - rule_width;
    if delim_depth > inner.height + inner.depth + clearance {
        clearance = (clearance + delim_depth - inner.height - inner.depth) / 2.0;
    }
    let image_shift = image.height - inner.height - clearance - rule_width;
    let kern = -(inner.height + image_shift);
    inner.set_style("paddingLeft", make_em(advance));
    let body = vlist(
        "firstBaseline",
        0.0,
        vec![
            elem(inner).classes(&["svg-align"]),
            VChild::Kern(kern),
            elem(image),
            VChild::Kern(rule_width),
        ],
    );
    if let Some(index) = index {
        let root = build_group_base(index, &o.with_style(6), o);
        let root_vlist = vlist("shift", -0.6 * (body.height - body.depth), vec![elem(root)]);
        span(
            &["mord", "sqrt"],
            vec![span(&["katex-root"], vec![root_vlist], None), body],
            Some(o),
        )
    } else {
        span(&["mord", "sqrt"], vec![body], Some(o))
    }
}
#[allow(clippy::too_many_arguments)]
fn layout_fraction(
    numer: &ParseNode,
    denom: &ParseNode,
    continued: bool,
    has_bar: bool,
    bar_size: Option<&Measurement>,
    left: Option<&str>,
    right: Option<&str>,
    mode: Mode,
    o: &Options,
) -> Node {
    let mut numerator = build_group_base(numer, &o.with_style(style_frac_num(o.style)), o);
    if continued {
        numerator.height = numerator.height.max(8.5 / o.metric("ptPerEm"));
        numerator.depth = numerator.depth.max(3.5 / o.metric("ptPerEm"));
    }
    let denominator = build_group_base(denom, &o.with_style(style_frac_den(o.style)), o);
    let rule = has_bar.then(|| line_span("frac-line", o, bar_size.map(|s| calculate_size(s, o))));
    let rw = rule.as_ref().map_or(0.0, |n| n.height);
    let rs = rule
        .as_ref()
        .map_or(o.metric("defaultRuleThickness"), |n| n.height);
    let mut num_shift = o.metric(if o.style / 2 == 0 {
        "num1"
    } else if rw > 0.0 {
        "num2"
    } else {
        "num3"
    });
    let clearance = if o.style / 2 == 0 {
        if rw > 0.0 {
            3.0
        } else {
            7.0
        }
    } else if rw > 0.0 {
        1.0
    } else {
        3.0
    } * rs;
    let mut den_shift = o.metric(if o.style / 2 == 0 { "denom1" } else { "denom2" });
    let fraction = if let Some(rule) = rule {
        let axis = o.metric("axisHeight");
        if num_shift - numerator.depth - (axis + 0.5 * rw) < clearance {
            num_shift += clearance - (num_shift - numerator.depth - (axis + 0.5 * rw));
        }
        if axis - 0.5 * rw - (denominator.height - den_shift) < clearance {
            den_shift += clearance - (axis - 0.5 * rw - (denominator.height - den_shift));
        }
        vlist(
            "individualShift",
            0.0,
            vec![
                shifted(denominator, den_shift),
                shifted(rule, -(axis - 0.5 * rw)),
                shifted(numerator, -num_shift),
            ],
        )
    } else {
        let candidate = num_shift - numerator.depth - (denominator.height - den_shift);
        if candidate < clearance {
            num_shift += 0.5 * (clearance - candidate);
            den_shift += 0.5 * (clearance - candidate);
        }
        vlist(
            "individualShift",
            0.0,
            vec![
                shifted(denominator, den_shift),
                shifted(numerator, -num_shift),
            ],
        )
    };
    let delim_size = if o.style / 2 == 0 {
        o.metric("delim1")
    } else if o.style / 2 == 3 {
        o.with_style(4).metric("delim2")
    } else {
        o.metric("delim2")
    };
    let left = left.map_or_else(
        || null_delimiter(o, classes(&["mopen"])),
        |d| custom_delim(d, delim_size, true, o, mode, classes(&["mopen"])),
    );
    let right = if continued {
        span(&[], vec![], None)
    } else {
        right.map_or_else(
            || null_delimiter(o, classes(&["mclose"])),
            |d| custom_delim(d, delim_size, true, o, mode, classes(&["mclose"])),
        )
    };
    span(
        &["mord"],
        vec![left, span(&["mfrac"], vec![fraction], None), right],
        Some(o),
    )
}
fn layout_line(body: &ParseNode, over: bool, o: &Options) -> Node {
    let inner = build_group(
        body,
        &if over {
            o.with_style(style_cramp(o.style))
        } else {
            o.clone()
        },
    );
    let line = line_span(
        if over {
            "overline-line"
        } else {
            "underline-line"
        },
        o,
        None,
    );
    let t = o.metric("defaultRuleThickness");
    let list = if over {
        vlist(
            "firstBaseline",
            0.0,
            vec![
                elem(inner),
                VChild::Kern(3.0 * t),
                elem(line),
                VChild::Kern(t),
            ],
        )
    } else {
        vlist(
            "top",
            inner.height,
            vec![
                VChild::Kern(t),
                elem(line),
                VChild::Kern(3.0 * t),
                elem(inner),
            ],
        )
    };
    span(
        &[
            "mord",
            if over {
                "katex-overline"
            } else {
                "katex-underline"
            },
        ],
        vec![list],
        Some(o),
    )
}
