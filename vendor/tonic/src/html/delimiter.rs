use super::*;
use geometry::{inner_delim_path, sqrt_path, tall_delim_path};
fn normalize(d: &str) -> &str {
    match d {
        "<" | "\\lt" | "⟨" => "\\langle",
        ">" | "\\gt" | "⟩" => "\\rangle",
        _ => d,
    }
}
fn sequence(d: &str) -> usize {
    match d {
        "<" | ">" | "\\langle" | "\\rangle" | "/" | "\\backslash" | "\\lt" | "\\gt" => 0,
        "(" | "\\lparen" | ")" | "\\rparen" | "[" | "\\lbrack" | "]" | "\\rbrack" | "\\{"
        | "\\lbrace" | "\\}" | "\\rbrace" | "\\lfloor" | "\\rfloor" | "⌊" | "⌋" | "\\lceil"
        | "\\rceil" | "⌈" | "⌉" | "\\surd" => 1,
        _ => 2,
    }
}
fn style_wrap(delim: Node, style: usize, o: &Options, mut names: Vec<String>) -> Node {
    let new = o.base_style(style);
    names.extend(new.sizing_classes(o));
    let mut n = span_c(names, vec![delim], Some(o));
    let ratio = new.size_multiplier / o.size_multiplier;
    n.height *= ratio;
    n.depth *= ratio;
    n.max_font_size = new.size_multiplier;
    n
}
fn center(n: &mut Node, o: &Options, style: usize) {
    let shift =
        (1.0 - o.size_multiplier / o.base_style(style).size_multiplier) * o.metric("axisHeight");
    n.classes.push("delimcenter".into());
    n.set_style("top", make_em(shift));
    n.height -= shift;
    n.depth += shift;
}
fn small(
    d: &str,
    style: usize,
    centered: bool,
    o: &Options,
    mode: Mode,
    names: Vec<String>,
) -> Node {
    let text = make_symbol(d, "Main-Regular", mode, o, vec![]);
    let mut n = style_wrap(text, style, o, names);
    if centered {
        center(&mut n, o, style);
    }
    n
}
fn large(
    d: &str,
    size: usize,
    centered: bool,
    o: &Options,
    mode: Mode,
    names: Vec<String>,
) -> Node {
    let inner = make_symbol(d, &format!("Size{size}-Regular"), mode, o, vec![]);
    let mut n = style_wrap(
        span(
            &["delimsizing", &format!("size{size}")],
            vec![inner],
            Some(o),
        ),
        2,
        o,
        names,
    );
    if centered {
        center(&mut n, o, 2);
    }
    n
}
fn glyph(s: &str, font: &str, mode: Mode, o: &Options) -> VChild {
    elem(span(
        &[
            "delimsizinginner",
            if font == "Size1-Regular" {
                "delim-size1"
            } else {
                "delim-size4"
            },
        ],
        vec![span(&[], vec![make_symbol(s, font, mode, o, vec![])], None)],
        None,
    ))
}
fn inner(ch: &str, height: f64, o: &Options) -> VChild {
    let g = make_symbol(ch, "Size4-Regular", Mode::Math, o, vec![]);
    let width = if g.width != 0.0 {
        g.width
    } else {
        make_symbol(ch, "Size1-Regular", Mode::Math, o, vec![]).width
    };
    let vh = (1000.0 * height).round();
    let image = svg(
        vec![path(inner_delim_path(ch, vh))],
        &[
            ("width", make_em(width)),
            ("height", make_em(height)),
            ("style", format!("width:{}", make_em(width))),
            (
                "viewBox",
                format!("0 0 {} {}", js_number(1000.0 * width), js_number(vh)),
            ),
            ("preserveAspectRatio", "xMinYMin".into()),
        ],
    );
    let mut n = svg_span(&[], vec![image], o);
    n.height = height;
    n.set_style("height", make_em(height));
    n.set_style("width", make_em(width));
    elem(n)
}
fn stacked(
    d: &str,
    height: f64,
    centered: bool,
    o: &Options,
    mode: Mode,
    names: Vec<String>,
) -> Node {
    let (top, middle, repeat, bottom, font, label, width) = match d {
        "\\uparrow" => (d, None, "⏐", "⏐", "Size1-Regular", "", 0.0),
        "\\Uparrow" => (d, None, "‖", "‖", "Size1-Regular", "", 0.0),
        "\\downarrow" => ("⏐", None, "⏐", d, "Size1-Regular", "", 0.0),
        "\\Downarrow" => ("‖", None, "‖", d, "Size1-Regular", "", 0.0),
        "\\updownarrow" => (
            "\\uparrow",
            None,
            "⏐",
            "\\downarrow",
            "Size1-Regular",
            "",
            0.0,
        ),
        "\\Updownarrow" => (
            "\\Uparrow",
            None,
            "‖",
            "\\Downarrow",
            "Size1-Regular",
            "",
            0.0,
        ),
        "|" | "\\lvert" | "\\rvert" | "\\vert" => (d, None, "∣", d, "Size1-Regular", "vert", 333.0),
        "\\|" | "\\lVert" | "\\rVert" | "\\Vert" => {
            (d, None, "∥", d, "Size1-Regular", "doublevert", 556.0)
        }
        "[" | "\\lbrack" => ("⎡", None, "⎢", "⎣", "Size4-Regular", "lbrack", 667.0),
        "]" | "\\rbrack" => ("⎤", None, "⎥", "⎦", "Size4-Regular", "rbrack", 667.0),
        "\\lfloor" | "⌊" => ("⎢", None, "⎢", "⎣", "Size4-Regular", "lfloor", 667.0),
        "\\lceil" | "⌈" => ("⎡", None, "⎢", "⎢", "Size4-Regular", "lceil", 667.0),
        "\\rfloor" | "⌋" => ("⎥", None, "⎥", "⎦", "Size4-Regular", "rfloor", 667.0),
        "\\rceil" | "⌉" => ("⎤", None, "⎥", "⎥", "Size4-Regular", "rceil", 667.0),
        "(" | "\\lparen" => ("⎛", None, "⎜", "⎝", "Size4-Regular", "lparen", 875.0),
        ")" | "\\rparen" => ("⎞", None, "⎟", "⎠", "Size4-Regular", "rparen", 875.0),
        "\\{" | "\\lbrace" => ("⎧", Some("⎨"), "⎪", "⎩", "Size4-Regular", "", 0.0),
        "\\}" | "\\rbrace" => ("⎫", Some("⎬"), "⎪", "⎭", "Size4-Regular", "", 0.0),
        "\\lgroup" | "⟮" => ("⎧", None, "⎪", "⎩", "Size4-Regular", "", 0.0),
        "\\rgroup" | "⟯" => ("⎫", None, "⎪", "⎭", "Size4-Regular", "", 0.0),
        "\\lmoustache" | "⎰" => ("⎧", None, "⎪", "⎭", "Size4-Regular", "", 0.0),
        "\\rmoustache" | "⎱" => ("⎫", None, "⎪", "⎩", "Size4-Regular", "", 0.0),
        _ => (d, None, d, d, "Size1-Regular", "", 0.0),
    };
    let total = |s| {
        let g = make_symbol(s, font, mode, o, vec![]);
        g.height + g.depth
    };
    let th = total(top);
    let rh = total(repeat);
    let bh = total(bottom);
    let mh = middle.map_or(0.0, total);
    let factor = if middle.is_some() { 2.0 } else { 1.0 };
    let min = th + bh + mh;
    let count = ((height - min) / (factor * rh)).ceil().max(0.0);
    let real = min + count * factor * rh;
    let axis = o.metric("axisHeight") * if centered { o.size_multiplier } else { 1.0 };
    let depth = real / 2.0 - axis;
    let mut stack = vec![];
    if !label.is_empty() {
        let vh = (real * 1000.0).round();
        let image = svg(
            vec![path(tall_delim_path(
                label,
                ((real - th - bh) * 1000.0).round(),
            ))],
            &[
                ("width", make_em(width / 1000.0)),
                ("height", make_em(vh / 1000.0)),
                (
                    "viewBox",
                    format!("0 0 {} {}", js_number(width), js_number(vh)),
                ),
            ],
        );
        let mut wrapper = svg_span(&[], vec![image], o);
        wrapper.height = vh / 1000.0;
        wrapper.set_style("width", make_em(width / 1000.0));
        wrapper.set_style("height", make_em(vh / 1000.0));
        stack.push(elem(wrapper));
    } else {
        stack.push(glyph(bottom, font, mode, o));
        stack.push(VChild::Kern(-0.008));
        if let Some(m) = middle {
            let h = (real - th - bh - mh) / 2.0 + 0.016;
            stack.push(inner(repeat, h, o));
            stack.push(VChild::Kern(-0.008));
            stack.push(glyph(m, font, mode, o));
            stack.push(VChild::Kern(-0.008));
            stack.push(inner(repeat, h, o));
        } else {
            stack.push(inner(repeat, real - th - bh + 0.016, o));
        }
        stack.push(VChild::Kern(-0.008));
        stack.push(glyph(top, font, mode, o));
    }
    let new = o.base_style(2);
    style_wrap(
        span(
            &["delimsizing", "mult"],
            vec![vlist("bottom", depth, stack)],
            Some(&new),
        ),
        2,
        o,
        names,
    )
}
fn choose(d: &str, height: f64, o: &Options) -> (&'static str, usize) {
    let seq = sequence(d);
    let start = (3 - o.style / 2).min(2);
    for i in start..if seq == 2 { 3 } else { 7 } {
        let font = if i < 3 {
            "Main-Regular".into()
        } else {
            format!("Size{}-Regular", i - 2)
        };
        let g = make_symbol(d, &font, Mode::Math, o, vec![]);
        let total = (g.height + g.depth)
            * if i < 3 {
                o.base_style(6 - 2 * i).size_multiplier
            } else {
                1.0
            };
        if total > height {
            return if i < 3 {
                ("small", 6 - 2 * i)
            } else {
                ("large", i - 2)
            };
        }
    }
    if seq == 0 {
        ("large", 4)
    } else {
        ("stack", 0)
    }
}
pub(super) fn sized_delim(
    d: &str,
    size: usize,
    o: &Options,
    mode: Mode,
    names: Vec<String>,
) -> Node {
    let d = normalize(d);
    if !(1..=4).contains(&size) {
        return Node::error("Invalid delimiter size");
    }
    if sequence(d) == 2 {
        stacked(d, [0.0, 1.2, 1.8, 2.4, 3.0][size], false, o, mode, names)
    } else {
        large(d, size, false, o, mode, names)
    }
}
pub(super) fn custom_delim(
    d: &str,
    height: f64,
    centered: bool,
    o: &Options,
    mode: Mode,
    names: Vec<String>,
) -> Node {
    let d = normalize(d);
    let (kind, size) = choose(d, height, o);
    match kind {
        "small" => small(d, size, centered, o, mode, names),
        "large" => large(d, size, centered, o, mode, names),
        _ => stacked(d, height, centered, o, mode, names),
    }
}
pub(super) fn left_right_delim(
    d: &str,
    height: f64,
    depth: f64,
    o: &Options,
    mode: Mode,
    names: Vec<String>,
) -> Node {
    let axis = o.metric("axisHeight") * o.size_multiplier;
    let dist = (height - axis).max(depth + axis);
    custom_delim(
        d,
        (dist / 500.0 * 901.0).max(2.0 * dist - 5.0 / o.metric("ptPerEm")),
        true,
        o,
        mode,
        names,
    )
}
pub(super) fn sqrt_image(height: f64, o: &Options) -> (Node, f64, f64) {
    let new = o.base_sizing();
    let (kind, size) = choose("\\surd", height * new.size_multiplier, &new);
    let mut multiplier = new.size_multiplier;
    let extra = (o.min_rule_thickness - o.metric("sqrtRuleThickness")).max(0.0);
    let (name, span_height, tex_height, vh, min_width, advance) = if kind == "small" {
        if height < 1.0 {
            multiplier = 1.0;
        } else if height < 1.4 {
            multiplier = 0.7;
        }
        (
            "sqrtMain".into(),
            (1.0 + extra + 0.08) / multiplier,
            (1.0 + extra) / multiplier,
            1000.0 + 1000.0 * extra + 80.0,
            "0.853em",
            0.833 / multiplier,
        )
    } else if kind == "large" {
        let h = [0.0, 1.2, 1.8, 2.4, 3.0][size];
        (
            format!("sqrtSize{size}"),
            (h + extra + 0.08) / multiplier,
            (h + extra) / multiplier,
            1080.0 * h,
            "1.02em",
            1.0 / multiplier,
        )
    } else {
        (
            "sqrtTall".into(),
            height + extra + 0.08,
            height + extra,
            (1000.0 * height + extra).floor() + 80.0,
            "0.742em",
            1.056,
        )
    };
    let image = svg(
        vec![path(sqrt_path(&name, extra, vh))],
        &[
            ("width", "400em".into()),
            ("height", make_em(span_height)),
            ("viewBox", format!("0 0 400000 {}", js_number(vh))),
            ("preserveAspectRatio", "xMinYMin slice".into()),
        ],
    );
    let mut n = svg_span(&["hide-tail"], vec![image], o);
    n.height = tex_height;
    n.set_style("minWidth", min_width);
    n.set_style("height", make_em(span_height));
    (
        n,
        (o.metric("sqrtRuleThickness") + extra) * multiplier,
        advance,
    )
}
