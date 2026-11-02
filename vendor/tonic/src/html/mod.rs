//! HTML layout compatible with KaTeX 0.19.0's `output: "html"`.
//!
//! Enable the `html` Cargo feature and load KaTeX's matching CSS and fonts in
//! the document displaying the result. Rendering runs entirely in Rust.
//!
//! Layout follows dulox/backend/html and KaTeX (see LICENSE-KaTeX).

use crate::{
    ArrayColumn, AtomFamily, ColumnSeparationType, LapAlignment, Measurement, Mode,
    OperatorContent, ParseError, ParseNode, StyleLevel,
};
mod accents;
mod array;
mod common;
mod data;
mod delimiter;
mod enclose;
mod expression;
mod extra;
mod geometry;
mod layout;
mod options;
mod scripts;
mod stretchy;
mod symbols;
mod tree;
use accents::*;
use array::*;
use common::*;
use delimiter::*;
use enclose::*;
use expression::*;
use extra::*;
use layout::*;
pub use options::RenderConfig;
use options::*;
use scripts::*;
use stretchy::*;
use symbols::*;
use tree::*;

/// Render parsed math as KaTeX HTML, including the `katex` wrapper.
///
/// Set `display_mode` consistently in the parser settings and this config.
/// Returns an error for non-renderable AST nodes or invalid HTML attributes.
///
/// ```
/// use katex_parser::{parse, render_html, HtmlRenderConfig, Settings};
/// let nodes = parse(r"\sqrt{x^2 + y^2}", &mut Settings::new())?;
/// let html = render_html(&nodes, HtmlRenderConfig::new())?;
/// # Ok::<(), katex_parser::ParseError>(())
/// ```
pub fn render(nodes: &[ParseNode], config: RenderConfig) -> Result<String, ParseError> {
    let mut options = Options::new(config);
    let nodes = if let [ParseNode::Styling {
        style: StyleLevel::DisplayStyle,
        reset_font: true,
        body,
        ..
    }] = nodes
    {
        options = options.with_style(0);
        body
    } else {
        nodes
    };
    let rendered = span(&["katex"], vec![build_html(nodes, &options)], None);
    if config.display_mode {
        let mut names = classes(&["katex-display"]);
        if config.leqno {
            names.push("leqno".into());
        }
        if config.fleqn {
            names.push("fleqn".into());
        }
        span_c(names, vec![rendered], None).to_markup()
    } else {
        rendered.to_markup()
    }
}
fn html_error(message: impl Into<String>) -> ParseError {
    ParseError::InvalidArgument {
        message: message.into(),
        loc: None,
    }
}
fn unbreakable(children: Vec<Node>, o: &Options) -> Node {
    let mut body = span(&["katex-base"], children, Some(o));
    let mut strut = span(&["katex-strut"], vec![], None);
    strut.set_style("height", make_em(body.height + body.depth));
    if body.depth != 0.0 {
        strut.set_style("vertical-align", make_em(-body.depth));
    }
    body.children.insert(0, strut);
    body
}
fn build_html(nodes: &[ParseNode], o: &Options) -> Node {
    let (nodes, tag) = if let [ParseNode::Tag { body, tag, .. }] = nodes {
        (body.as_slice(), Some(tag))
    } else {
        (nodes, None)
    };
    let mut expression = expression(nodes, o, true, true, "leftmost", "rightmost");
    let equation_number = if expression.len() == 2 && expression[1].has_class("katex-tag") {
        expression.pop()
    } else {
        None
    };
    let mut children = vec![];
    let mut parts = vec![];
    let mut iter = expression.into_iter().peekable();
    while let Some(node) = iter.next() {
        let can_break =
            node.has_class("mbin") || node.has_class("mrel") || node.has_class("allowbreak");
        if node.has_class("katex-newline") {
            if !parts.is_empty() {
                children.push(unbreakable(std::mem::take(&mut parts), o));
            }
            children.push(node);
        } else {
            parts.push(node);
            if can_break {
                let mut prevent = false;
                while iter
                    .peek()
                    .is_some_and(|n| n.has_class("mspace") && !n.has_class("katex-newline"))
                {
                    let n = iter.next().unwrap();
                    prevent |= n.has_class("nobreak");
                    parts.push(n);
                }
                if !prevent {
                    children.push(unbreakable(std::mem::take(&mut parts), o));
                }
            }
        }
    }
    if !parts.is_empty() {
        children.push(unbreakable(parts, o));
    }
    if let Some(tag) = tag {
        let mut child = unbreakable(build_expression(tag, o), o);
        child.classes = classes(&["katex-tag"]);
        children.push(child);
    } else if let Some(number) = equation_number {
        children.push(number);
    }
    let mut html = span(&["katex-html"], children, None);
    html.set_attribute("aria-hidden", "true");
    if tag.is_some() {
        let strut = &mut html.children.last_mut().unwrap().children[0];
        strut.set_style("height", make_em(html.height + html.depth));
        if html.depth != 0.0 {
            strut.set_style("vertical-align", make_em(-html.depth));
        }
    }
    html
}
fn build_group(node: &ParseNode, o: &Options) -> Node {
    build_node(node, o)
}
fn build_group_base(node: &ParseNode, o: &Options, base: &Options) -> Node {
    let group = build_node(node, o);
    if o.size == base.size {
        return group;
    }
    let mut sized = span_c(o.sizing_classes(base), vec![group], Some(o));
    let ratio = o.size_multiplier / base.size_multiplier;
    sized.height *= ratio;
    sized.depth *= ratio;
    sized
}
fn atom_class(family: AtomFamily) -> String {
    crate::atom_family_name(family)
}
fn null_delimiter(o: &Options, mut names: Vec<String>) -> Node {
    names.push("nulldelimiter".into());
    names.extend(o.base_sizing_classes());
    span_c(names, vec![], None)
}
fn build_node(node: &ParseNode, o: &Options) -> Node {
    use ParseNode::*;
    match node {
        Unsupported {
            mode,
            command,
            color,
            ..
        } => build_group(
            &crate::parser::format_unsupported_command(*mode, color, command),
            o,
        ),
        MathOrd { .. } | TextOrd { .. } => make_ord(node, o),
        Atom {
            text, mode, family, ..
        } => math_symbol(text, *mode, o, vec![atom_class(*family)]),
        OrdGroup {
            body, semisimple, ..
        } => {
            let inner = expression(body, o, !semisimple, false, "leftmost", "rightmost");
            if *semisimple {
                fragment(inner)
            } else {
                span(&["mord"], inner, Some(o))
            }
        }
        Color { body, color, .. } => fragment(partial_expression(
            body,
            &Options {
                color: Some(color.clone()),
                ..o.clone()
            },
        )),
        Sizing { body, size, .. } => {
            if (1..=11).contains(size) {
                sizing_group(body, &o.with_size(*size), o)
            } else {
                Node::error("Invalid font size")
            }
        }
        Styling {
            body,
            style,
            reset_font,
            ..
        } => {
            let mut styled = o.with_style(style_id(*style));
            if *reset_font {
                styled.font.clear();
            }
            sizing_group(body, &styled, o)
        }
        Font { body, font, .. } => build_group(
            body,
            &Options {
                font: font.clone(),
                ..o.clone()
            },
        ),
        Text { body, font, .. } => {
            let new = text_options(font, o);
            span(&["mord", "text"], build_expression(body, &new), Some(&new))
        }
        MClass { body, mclass, .. } => span_c(
            vec![atom_class(*mclass)],
            build_expression(body, o),
            Some(o),
        ),
        HtmlMathML { html, .. } => fragment(partial_expression(html, o)),
        MathChoice {
            display,
            text,
            script,
            scriptscript,
            ..
        } => fragment(partial_expression(
            match o.style / 2 {
                0 => display,
                1 => text,
                2 => script,
                _ => scriptscript,
            },
            o,
        )),
        Kern { dimension, .. } => glue(dimension, o),
        Spacing { text, mode, .. } => {
            if text == "\\nobreak" {
                span(&["mspace", "nobreak"], vec![], Some(o))
            } else if text == "\\allowbreak" {
                span(&["mspace", "allowbreak"], vec![], Some(o))
            } else {
                let name = if matches!(text.as_str(), "~" | "\\nobreakspace") {
                    "nobreak"
                } else {
                    ""
                };
                if *mode == Mode::Text {
                    let mut n = make_ord(node, o);
                    n.classes.push(name.into());
                    n
                } else {
                    span(
                        &["mspace", name],
                        vec![math_symbol(text, *mode, o, vec![])],
                        Some(o),
                    )
                }
            }
        }
        Href { href, body, .. } => {
            let mut n = span(&[], partial_expression(body, o), Some(o));
            n.kind = "a";
            n.set_attribute("href", href);
            n
        }
        Html {
            attributes, body, ..
        } => {
            let mut names = classes(&["enclosing"]);
            if let Some((_, c)) = attributes.iter().find(|(k, _)| k == "class") {
                names.extend(c.split_whitespace().map(str::to_string));
            }
            let mut n = span_c(names, partial_expression(body, o), Some(o));
            for (k, v) in attributes {
                if k != "class" {
                    n.set_attribute(k, v);
                }
            }
            n
        }
        Internal { .. } | EnvironmentEnd { .. } => fragment(vec![]),
        _ => layout_node(node, o)
            .or_else(|| extra_node(node, o))
            .unwrap_or_else(|| Node::error("Unexpected non-renderable KaTeX parse node")),
    }
}
fn sizing_group(nodes: &[ParseNode], o: &Options, base: &Options) -> Node {
    let mut inner = partial_expression(nodes, o);
    let ratio = o.size_multiplier / base.size_multiplier;
    for n in &mut inner {
        if let Some(pos) = n.classes.iter().position(|c| c == "katex-sizing") {
            if n.classes.get(pos + 1) == Some(&format!("reset-size{}", o.size)) {
                n.classes[pos + 1] = format!("reset-size{}", base.size);
            }
        } else {
            n.classes.extend(o.sizing_classes(base));
        }
        n.height *= ratio;
        n.depth *= ratio;
    }
    fragment(inner)
}
fn text_options(font: &str, o: &Options) -> Options {
    let mut n = o.clone();
    match font {
        "\\textrm" | "\\textnormal" => n.font_family = "textrm".into(),
        "\\textsf" => n.font_family = "textsf".into(),
        "\\texttt" => n.font_family = "texttt".into(),
        "\\textbf" => n.font_weight = "textbf".into(),
        "\\textmd" => n.font_weight = "textmd".into(),
        "\\textit" => n.font_shape = "textit".into(),
        "\\textup" => n.font_shape = "textup".into(),
        "\\emph" => {
            n.font_shape = if o.font_shape == "textit" {
                "textup"
            } else {
                "textit"
            }
            .into()
        }
        _ => return n,
    }
    n.font.clear();
    n
}
fn glue(size: &Measurement, o: &Options) -> Node {
    let mut n = span(&["mspace"], vec![], Some(o));
    n.set_style("margin-right", make_em(calculate_size(size, o)));
    n
}
