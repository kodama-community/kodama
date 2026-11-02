use super::*;
use geometry::{stretchy_image_data, svg_path};
pub(super) fn static_svg(value: &str, o: &Options) -> Node {
    let (width, height) = match value {
        "vec" => (0.471, 0.714),
        "oiintSize1" => (0.957, 0.499),
        "oiintSize2" => (1.472, 0.659),
        "oiiintSize1" => (1.304, 0.499),
        "oiiintSize2" => (1.98, 0.659),
        _ => (0.0, 0.0),
    };
    let image = svg(
        vec![path(svg_path(value))],
        &[
            ("width", make_em(width)),
            ("height", make_em(height)),
            ("style", format!("width:{}", make_em(width))),
            (
                "viewBox",
                format!(
                    "0 0 {} {}",
                    js_number(1000.0 * width),
                    js_number(1000.0 * height)
                ),
            ),
            ("preserveAspectRatio", "xMinYMin".into()),
        ],
    );
    let mut n = svg_span(&["katex-overlay"], vec![image], o);
    n.height = height;
    n.set_style("height", make_em(height));
    n.set_style("width", make_em(width));
    n
}
pub(super) fn stretchy_svg(label: &str, base: Option<&ParseNode>, o: &Options) -> Node {
    let label = label.strip_prefix('\\').unwrap_or(label);
    if matches!(label, "widehat" | "widecheck" | "widetilde" | "utilde") {
        let count = if let Some(ParseNode::OrdGroup { body, .. }) = base {
            body.len()
        } else {
            1
        };
        let hat = matches!(label, "widehat" | "widecheck");
        let index = if count > 5 {
            4
        } else {
            [1, 1, 2, 2, 3, 3][count]
        };
        let vw = if hat {
            [0.0, 1062.0, 2364.0, 2364.0, 2364.0][index]
        } else {
            [0.0, 600.0, 1033.0, 2339.0, 2340.0][index]
        };
        let vh = if hat {
            [0.0, 239.0, 300.0, 360.0, 420.0][index]
        } else {
            [0.0, 260.0, 286.0, 306.0, 312.0][index]
        };
        let height = if count > 5 {
            if hat {
                0.42
            } else {
                0.34
            }
        } else if hat {
            [0.0, 0.24, 0.3, 0.3, 0.36, 0.42][index]
        } else {
            [0.0, 0.26, 0.286, 0.3, 0.306, 0.34][index]
        };
        let name = format!("{}{index}", if hat { label } else { "tilde" });
        let image = svg(
            vec![path(svg_path(&name))],
            &[
                ("width", "100%".into()),
                ("height", make_em(height)),
                (
                    "viewBox",
                    format!("0 0 {} {}", js_number(vw), js_number(vh)),
                ),
                ("preserveAspectRatio", "none".into()),
            ],
        );
        let mut n = svg_span(&[], vec![image], o);
        n.height = height;
        n.set_style("height", make_em(height));
        return n;
    }
    let (paths, min_width, vh, align) = stretchy_image_data(label);
    let height = vh / 1000.0;
    let count = paths.len();
    let (names, aligns): (&[&str], &[&str]) = match count {
        1 => (&["hide-tail"], &[align]),
        2 => (
            &["halfarrow-left", "halfarrow-right"],
            &["xMinYMin", "xMaxYMin"],
        ),
        _ => (
            &["brace-left", "brace-center", "brace-right"],
            &["xMinYMin", "xMidYMin", "xMaxYMin"],
        ),
    };
    let mut children = vec![];
    for i in 0..count {
        let image = svg(
            vec![path(svg_path(paths[i]))],
            &[
                ("width", "400em".into()),
                ("height", make_em(height)),
                ("viewBox", format!("0 0 400000 {}", js_number(vh))),
                ("preserveAspectRatio", format!("{} slice", aligns[i])),
            ],
        );
        let mut child = svg_span(&[names[i]], vec![image], o);
        if count > 1 {
            child.set_style("height", make_em(height));
        }
        children.push(child);
    }
    let mut n = if count == 1 {
        children.remove(0)
    } else {
        span(&["katex-stretchy"], children, Some(o))
    };
    n.height = height;
    n.set_style("height", make_em(height));
    if min_width > 0.0 {
        n.set_style("minWidth", make_em(min_width));
    }
    n
}
