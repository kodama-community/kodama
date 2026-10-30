use super::*;
fn cancel_line(descending: bool) -> Node {
    let mut n = Node::new("line", vec![], vec![]);
    n.set_attribute("x1", "0");
    n.set_attribute("y1", if descending { "0" } else { "100%" });
    n.set_attribute("x2", "100%");
    n.set_attribute("y2", if descending { "100%" } else { "0" });
    n.set_attribute("stroke-width", "0.046em");
    n
}
fn stretchy_enclose(inner: &Node, label: &str, top: f64, bottom: f64, o: &Options) -> Node {
    let height = inner.height + inner.depth + top + bottom;
    let mut n = if label == "fbox" || label.contains("color") || label == "angl" {
        let mut n = span(&["katex-stretchy", label], vec![], Some(o));
        if label == "fbox" && o.color.is_some() {
            if let Some(color) = o.get_color() {
                n.set_style("borderColor", color);
            }
        }
        n
    } else {
        let mut lines = vec![];
        if matches!(label, "bcancel" | "xcancel") {
            lines.push(cancel_line(true));
        }
        if matches!(label, "cancel" | "xcancel") {
            lines.push(cancel_line(false));
        }
        span(
            &[],
            vec![svg(
                lines,
                &[("width", "100%".into()), ("height", make_em(height))],
            )],
            Some(o),
        )
    };
    n.height = height;
    n.set_style("height", make_em(height));
    n
}
pub(super) fn enclose(
    command: &str,
    body: &ParseNode,
    background: Option<&str>,
    border: Option<&str>,
    o: &Options,
) -> Node {
    let mut inner = wrap_fragment(build_group(body, o), o);
    let label = command.strip_prefix('\\').unwrap_or(command);
    let single = character_box(body);
    let (image, shift) = if label == "sout" {
        let mut image = span(&["katex-stretchy", "katex-sout"], vec![], None);
        image.height = o.metric("defaultRuleThickness") / o.size_multiplier;
        (image, -0.5 * o.metric("xHeight"))
    } else if label == "phase" {
        let weight = calculate_size(
            &Measurement {
                number: 0.6,
                unit: "pt".into(),
            },
            o,
        );
        let clearance = calculate_size(
            &Measurement {
                number: 0.35,
                unit: "ex".into(),
            },
            o,
        );
        let scale = o.size_multiplier / o.base_sizing().size_multiplier;
        let height = inner.height + inner.depth + weight + clearance;
        inner.set_style("paddingLeft", make_em(height / 2.0 + weight));
        let vh = (1000.0 * height * scale).floor();
        let image = svg(
            vec![path(geometry::phase_path(vh))],
            &[
                ("width", "400em".into()),
                ("height", make_em(vh / 1000.0)),
                ("viewBox", format!("0 0 400000 {}", js_number(vh))),
                ("preserveAspectRatio", "xMinYMin slice".into()),
            ],
        );
        let mut image = span(&["hide-tail"], vec![image], Some(o));
        image.set_style("height", make_em(height));
        (image, inner.depth + weight + clearance)
    } else {
        if label.contains("cancel") {
            if !single {
                inner.classes.push("cancel-pad".into());
            }
        } else if label == "angl" {
            inner.classes.push("anglpad".into());
        } else {
            inner.classes.push("boxpad".into());
        }
        let (top, bottom, rule) = if label.contains("box") {
            let r = o.metric("fboxrule").max(o.min_rule_thickness);
            let p = o.metric("fboxsep") + if label == "colorbox" { 0.0 } else { r };
            (p, p, r)
        } else if label == "angl" {
            let r = o.metric("defaultRuleThickness").max(o.min_rule_thickness);
            (4.0 * r, (0.25 - inner.depth).max(0.0), r)
        } else {
            let p = if single { 0.2 } else { 0.0 };
            (p, p, 0.0)
        };
        let mut image = stretchy_enclose(&inner, label, top, bottom, o);
        if matches!(label, "fbox" | "boxed" | "fcolorbox") {
            image.set_style("borderStyle", "solid");
            image.set_style("borderWidth", make_em(rule));
        } else if label == "angl" && rule != 0.049 {
            image.set_style("borderTopWidth", make_em(rule));
            image.set_style("borderRightWidth", make_em(rule));
        }
        if let Some(bg) = background {
            image.set_style("backgroundColor", bg);
            if let Some(b) = border {
                image.set_style("borderColor", b);
            }
        }
        (image, inner.depth + bottom)
    };
    let (h, d) = (inner.height, inner.depth);
    let children = if background.is_some() {
        vec![shifted(image, shift), elem(inner)]
    } else {
        vec![
            elem(inner),
            shifted(image, shift).classes(if label.contains("cancel") || label == "phase" {
                &["svg-align"]
            } else {
                &[]
            }),
        ]
    };
    let mut list = vlist("individualShift", 0.0, children);
    if label.contains("cancel") {
        list.height = h;
        list.depth = d;
    }
    span(
        if label.contains("cancel") && !single {
            &["mord", "cancel-lap"]
        } else {
            &["mord"]
        },
        vec![list],
        Some(o),
    )
}
