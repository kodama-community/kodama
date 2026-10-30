use super::*;
pub(super) fn extra_node(node: &ParseNode, o: &Options) -> Option<Node> {
    use ParseNode::*;
    Some(match node {
        Phantom { body, .. } => fragment(partial_expression(
            body,
            &Options {
                phantom: true,
                ..o.clone()
            },
        )),
        VPhantom { body, .. } => {
            let inner = span(
                &["katex-inner"],
                vec![build_group(
                    body,
                    &Options {
                        phantom: true,
                        ..o.clone()
                    },
                )],
                None,
            );
            span(
                &["mord", "rlap"],
                vec![inner, span(&["katex-fix"], vec![], None)],
                Some(o),
            )
        }
        Smash {
            body,
            smash_height,
            smash_depth,
            ..
        } => smash(body, *smash_height, *smash_depth, o),
        VCenter { body, .. } => {
            let b = build_group(body, o);
            let axis = o.metric("axisHeight");
            vlist(
                "shift",
                0.5 * (b.height - axis - (b.depth + axis)),
                vec![elem(b)],
            )
        }
        RaiseBox { dy, body, .. } => vlist(
            "shift",
            -calculate_size(dy, o),
            vec![elem(build_group(body, o))],
        ),
        ReflectBox { body, .. } => {
            span(&["mord", "reflectbox"], vec![build_group(body, o)], Some(o))
        }
        HBox { body, .. } => fragment(partial_expression(
            body,
            &Options {
                font: String::new(),
                ..o.clone()
            },
        )),
        Lap {
            alignment, body, ..
        } => lap(*alignment, body, o),
        Pmb { mclass, body, .. } => {
            let mut n = span_c(
                vec![atom_class(*mclass)],
                build_expression(body, o),
                Some(o),
            );
            n.set_style("textShadow", "0.02em 0.01em 0.04px");
            n
        }
        Rule {
            width,
            height,
            shift,
            ..
        } => {
            let mut n = span(&["mord", "katex-rule"], vec![], Some(o));
            let w = calculate_size(width, o);
            let h = calculate_size(height, o);
            let s = shift.as_ref().map_or(0.0, |v| calculate_size(v, o));
            n.set_style("borderRightWidth", make_em(w));
            n.set_style("borderTopWidth", make_em(h));
            n.set_style("bottom", make_em(s));
            n.width = w;
            n.height = h + s;
            n.depth = -s;
            n.max_font_size = h * 1.125 * o.size_multiplier;
            n
        }
        IncludeGraphics {
            alt,
            width,
            height,
            totalheight,
            src,
            ..
        } => {
            let h = calculate_size(height, o);
            let d = if totalheight.number > 0.0 {
                calculate_size(totalheight, o) - h
            } else {
                0.0
            };
            let w = if width.number > 0.0 {
                calculate_size(width, o)
            } else {
                0.0
            };
            let mut n = Node::new("img", classes(&["mord"]), vec![]);
            n.set_attribute("src", src);
            n.set_attribute("alt", alt);
            n.set_style("height", make_em(h + d));
            if w > 0.0 {
                n.set_style("width", make_em(w));
            }
            if d > 0.0 {
                n.set_style("verticalAlign", make_em(-d));
            }
            n.height = h;
            n.depth = d;
            n
        }
        CdLabel { side, label, .. } => {
            let mut n = wrap_fragment(
                build_group_base(label, &o.with_style(style_sup(o.style)), o),
                o,
            );
            n.classes.push(format!("cd-label-{side}"));
            n.set_style("bottom", make_em(0.8 - n.depth));
            n.height = 0.0;
            n.depth = 0.0;
            n
        }
        CdParent { fragment, .. } => {
            let mut n = wrap_fragment(build_group(fragment, o), o);
            n.classes.push("cd-vert-arrow".into());
            n
        }
        Cr { new_line, size, .. } => {
            let mut n = span(&["mspace"], vec![], Some(o));
            if *new_line {
                n.classes.push("katex-newline".into());
                if let Some(v) = size {
                    n.set_style("marginTop", make_em(calculate_size(v, o)));
                }
            }
            n
        }
        Verb {
            body, star, mode, ..
        } => {
            let new = o.with_style(style_text(o.style));
            let mut symbols = body
                .chars()
                .map(|c| {
                    let t = match c {
                        ' ' => {
                            if *star {
                                "␣".into()
                            } else {
                                "\u{a0}".into()
                            }
                        }
                        '~' => "\\textasciitilde".into(),
                        _ => c.to_string(),
                    };
                    make_symbol(
                        &t,
                        "Typewriter-Regular",
                        *mode,
                        &new,
                        classes(&["mord", "texttt"]),
                    )
                })
                .collect::<Vec<_>>();
            combine_chars(&mut symbols);
            let mut names = classes(&["mord", "text"]);
            names.extend(new.sizing_classes(o));
            span_c(names, symbols, Some(&new))
        }
        Enclose {
            label,
            body,
            background_color,
            border_color,
            ..
        } => enclose(
            label,
            body,
            background_color.as_deref(),
            border_color.as_deref(),
            o,
        ),
        Array { .. } => array(node, o),
        _ => return None,
    })
}
fn smash(body: &ParseNode, height: bool, depth: bool, o: &Options) -> Node {
    let mut n = span(&[], vec![build_group(body, o)], None);
    if !height && !depth {
        return n;
    }
    if height {
        n.height = 0.0;
    }
    if depth {
        n.depth = 0.0;
    }
    if height && depth {
        return span(&["mord", "katex-smash"], vec![n], Some(o));
    }
    for c in &mut n.children {
        if height {
            c.height = 0.0;
        }
        if depth {
            c.depth = 0.0;
        }
    }
    span(
        &["mord"],
        vec![vlist("firstBaseline", 0.0, vec![elem(n)])],
        Some(o),
    )
}
fn lap(alignment: LapAlignment, body: &ParseNode, o: &Options) -> Node {
    let name = match alignment {
        LapAlignment::LLap => "llap",
        LapAlignment::RLap => "rlap",
        LapAlignment::CLap => "clap",
    };
    let body = build_group(body, o);
    let inner = if alignment == LapAlignment::CLap {
        span(&["katex-inner"], vec![span(&[], vec![body], None)], Some(o))
    } else {
        span(&["katex-inner"], vec![body], None)
    };
    let mut n = span(
        &[name],
        vec![inner, span(&["katex-fix"], vec![], None)],
        Some(o),
    );
    let mut strut = span(&["katex-strut"], vec![], None);
    strut.set_style("height", make_em(n.height + n.depth));
    if n.depth != 0.0 {
        strut.set_style("verticalAlign", make_em(-n.depth));
    }
    n.children.insert(0, strut);
    span(
        &["mord", "katex-vbox"],
        vec![span(&["katex-thinbox"], vec![n], Some(o))],
        Some(o),
    )
}
