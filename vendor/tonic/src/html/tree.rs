use super::*;

#[derive(Clone, Debug)]
pub(super) struct Node {
    pub kind: &'static str,
    pub classes: Vec<String>,
    pub children: Vec<Node>,
    pub text: String,
    pub height: f64,
    pub depth: f64,
    pub max_font_size: f64,
    pub italic: f64,
    pub skew: f64,
    pub width: f64,
    pub style: Vec<(String, String)>,
    pub attributes: Vec<(String, String)>,
    pub middle: Option<(String, Options)>,
}
impl Node {
    pub fn new(kind: &'static str, classes: Vec<String>, children: Vec<Node>) -> Self {
        Self {
            kind,
            classes,
            children,
            text: String::new(),
            height: 0.0,
            depth: 0.0,
            max_font_size: 0.0,
            italic: 0.0,
            skew: 0.0,
            width: 0.0,
            style: vec![],
            attributes: vec![],
            middle: None,
        }
    }
    pub fn text(kind: &'static str, text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ..Self::new(kind, vec![], vec![])
        }
    }
    pub fn error(text: impl Into<String>) -> Self {
        Self::text("error", text)
    }
    pub fn set_style(&mut self, key: &str, value: impl Into<String>) {
        set_pair(&mut self.style, key, value.into());
    }
    pub fn set_attribute(&mut self, key: &str, value: impl Into<String>) {
        set_pair(&mut self.attributes, key, value.into());
    }
    pub fn has_class(&self, value: &str) -> bool {
        self.classes.iter().any(|c| c == value)
    }
    pub fn class(&self) -> &str {
        self.classes.first().map_or("", String::as_str)
    }
    pub fn size_from_children(&mut self) {
        for c in &self.children {
            self.height = self.height.max(c.height);
            self.depth = self.depth.max(c.depth);
            self.max_font_size = self.max_font_size.max(c.max_font_size);
        }
    }
    pub fn to_markup(&self) -> Result<String, ParseError> {
        let mut out = String::new();
        self.write_markup(&mut out)?;
        Ok(out)
    }
    fn write_markup(&self, out: &mut String) -> Result<(), ParseError> {
        use std::fmt::Write;
        if self.kind == "error" {
            return Err(html_error(&self.text));
        }
        if self.kind == "fragment" {
            for c in &self.children {
                c.write_markup(out)?;
            }
            return Ok(());
        }
        if self.kind == "path" {
            write!(out, "<path d=\"{}\"/>", escape(&self.text)).unwrap();
            return Ok(());
        }
        if self.kind == "img" {
            let attr = |name| {
                self.attributes
                    .iter()
                    .find(|(k, _)| k == name)
                    .map_or("", |(_, v)| v.as_str())
            };
            write!(
                out,
                "<img src=\"{}\" alt=\"{}\"",
                escape(attr("src")),
                escape(attr("alt"))
            )
            .unwrap();
            let styles = css_string(&self.style);
            if !styles.is_empty() {
                write!(out, " style=\"{}\"", escape(&styles)).unwrap();
            }
            out.push_str("'/>");
            return Ok(());
        }
        let mut styles = String::new();
        if self.kind == "symbol" && self.italic > 0.0 {
            write!(styles, "margin-right:{};", make_em(self.italic)).unwrap();
        }
        styles.push_str(&css_string(&self.style));
        if self.kind == "symbol" && self.classes.is_empty() && styles.is_empty() {
            out.push_str(&escape(&self.text));
            return Ok(());
        }
        let tag = if self.kind == "symbol" {
            "span"
        } else {
            self.kind
        };
        write!(out, "<{tag}").unwrap();
        if tag == "svg" {
            out.push_str(" xmlns=\"http://www.w3.org/2000/svg\"");
        }
        if !self.classes.is_empty() {
            write!(out, " class=\"{}\"", escape(&class_string(&self.classes))).unwrap();
        }
        if !styles.is_empty() {
            write!(out, " style=\"{}\"", escape(&styles)).unwrap();
        }
        for (key, value) in &self.attributes {
            if key.chars().any(|c| {
                c <= '\u{20}'
                    || matches!(c, '"' | '\'' | '>' | '/' | '=' | '\u{feff}')
                    || c.is_whitespace()
            }) {
                return Err(html_error(format!("Invalid attribute name '{key}'")));
            }
            write!(out, " {key}=\"{}\"", escape(value)).unwrap();
        }
        if tag == "line" {
            out.push_str("/>");
            return Ok(());
        }
        out.push('>');
        if self.kind == "symbol" {
            out.push_str(&escape(&self.text));
        }
        for c in &self.children {
            c.write_markup(out)?;
        }
        write!(out, "</{tag}>").unwrap();
        Ok(())
    }
}
fn set_pair(entries: &mut Vec<(String, String)>, key: &str, value: String) {
    if let Some(entry) = entries.iter_mut().find(|(k, _)| k == key) {
        entry.1 = value;
    } else {
        entries.push((key.into(), value));
    }
}
pub(super) fn escape(text: &str) -> String {
    let mut out = String::new();
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            _ => out.push(c),
        }
    }
    out
}
fn css_string(style: &[(String, String)]) -> String {
    let mut out = String::new();
    for (key, value) in style {
        for c in key.chars() {
            if c.is_ascii_uppercase() {
                out.push('-');
                out.push(c.to_ascii_lowercase());
            } else {
                out.push(c);
            }
        }
        out.push(':');
        out.push_str(value);
        out.push(';');
    }
    out
}
pub(super) fn classes(names: &[&str]) -> Vec<String> {
    names.iter().map(|s| (*s).into()).collect()
}
pub(super) fn class_string(names: &[String]) -> String {
    names
        .iter()
        .filter(|s| !s.is_empty())
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join(" ")
}
pub(super) fn span(names: &[&str], children: Vec<Node>, o: Option<&Options>) -> Node {
    span_c(classes(names), children, o)
}
pub(super) fn span_c(names: Vec<String>, children: Vec<Node>, o: Option<&Options>) -> Node {
    let mut n = Node::new("span", names, children);
    if let Some(o) = o {
        if o.style >= 4 {
            n.classes.push("mtight".into());
        }
        if let Some(color) = o.get_color() {
            n.set_style("color", color);
        }
    }
    n.size_from_children();
    n
}
pub(super) fn fragment(children: Vec<Node>) -> Node {
    let mut n = Node::new("fragment", vec![], children);
    n.size_from_children();
    n
}
pub(super) fn svg(children: Vec<Node>, attrs: &[(&str, String)]) -> Node {
    let mut n = Node::new("svg", vec![], children);
    for (k, v) in attrs {
        n.set_attribute(k, v);
    }
    n
}
pub(super) fn path(data: impl Into<String>) -> Node {
    Node::text("path", data)
}
pub(super) fn svg_span(names: &[&str], children: Vec<Node>, o: &Options) -> Node {
    let mut n = span(names, children, Some(o));
    n.height = 0.0;
    n.depth = 0.0;
    n.max_font_size = 0.0;
    n
}
