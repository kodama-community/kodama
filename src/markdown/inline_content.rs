// Copyright (c) 2025 Kodama Project. All rights reserved.
// Released under the GPL-3.0 license as described in the file LICENSE.
// Authors: Kokic (@kokic), Spore (@s-cerevisiae)

use pulldown_cmark::{Event, Tag, TagEnd};
use pulldown_cmark_escape::{escape_html, escape_html_body_text};

/// Escaping strategy when rendering inline content to a string.
#[derive(Clone, Copy)]
pub enum Escape {
    /// Keep text as-is. Used for raw typst source and for collection that is
    /// escaped once at a later stage.
    Raw,
    /// Escape for HTML text content, e.g. inside `<figcaption>`.
    Body,
    /// Escape for HTML attribute values, e.g. `alt` / `title`.
    Attribute,
}

/// Returns `true` for start tags of inline formatting that is valid inside
/// link texts and captions.
pub fn is_formatting_start(tag: &Tag) -> bool {
    matches!(
        tag,
        Tag::Emphasis | Tag::Strong | Tag::Strikethrough | Tag::Superscript | Tag::Subscript
    )
}

/// Returns `true` for end tags matching inline formatting.
pub fn is_formatting_end(tag: &TagEnd) -> bool {
    matches!(
        tag,
        TagEnd::Emphasis
            | TagEnd::Strong
            | TagEnd::Strikethrough
            | TagEnd::Superscript
            | TagEnd::Subscript
    )
}

/// Emits a warning for an event that is not valid inside a link text or caption.
pub fn warn_unexpected(what: &str, context: &str) {
    color_print::ceprintln!(
        "<y>Warning: unexpected {what} inside {context}; it will be ignored.</>"
    );
}

/// Writes a single leaf inline event into `out` following raw-text semantics:
/// formatting is already consumed (and stripped) by the caller, inline math is
/// kept in `$..$` form for KaTeX, and soft/hard line breaks collapse to a
/// single space.
pub fn write_leaf(out: &mut String, event: Event<'_>, escape: Escape, code_literal: bool) {
    use Event::*;
    match event {
        Text(text) => write_escaped(out, &text, escape),
        Code(text) => {
            if code_literal {
                out.push_str("<code>");
                write_escaped(out, &text, escape);
                out.push_str("</code>");
            } else {
                write_escaped(out, &text, escape);
            }
        }
        InlineMath(text) => {
            out.push('$');
            write_escaped(out, &text, escape);
            out.push('$');
        }
        DisplayMath(text) => {
            out.push_str("$$");
            write_escaped(out, &text, escape);
            out.push_str("$$");
        }
        InlineHtml(html) => write_escaped(out, &html, escape),
        SoftBreak | HardBreak | Rule => out.push(' '),
        _ => {}
    }
}

fn write_escaped(out: &mut String, text: &str, escape: Escape) {
    match escape {
        Escape::Raw => out.push_str(text),
        Escape::Body => escape_html_body_text(out, text).unwrap(),
        Escape::Attribute => escape_html(out, text).unwrap(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_leaf_math_kept_with_delimiters() {
        let mut out = String::new();
        write_leaf(
            &mut out,
            Event::InlineMath("x<y".into()),
            Escape::Raw,
            false,
        );
        assert_eq!(out, "$x<y$");
        out.clear();
        write_leaf(
            &mut out,
            Event::InlineMath("x<y".into()),
            Escape::Body,
            false,
        );
        assert_eq!(out, "$x&lt;y$");
    }

    #[test]
    fn test_write_leaf_code_literal_and_escaped() {
        let mut out = String::new();
        write_leaf(&mut out, Event::Code("a<b".into()), Escape::Raw, true);
        assert_eq!(out, "<code>a<b</code>");
        out.clear();
        write_leaf(&mut out, Event::Code("a<b".into()), Escape::Body, true);
        assert_eq!(out, "<code>a&lt;b</code>");
        out.clear();
        write_leaf(&mut out, Event::Code("a<b".into()), Escape::Body, false);
        assert_eq!(out, "a&lt;b");
    }

    #[test]
    fn test_write_leaf_breaks_collapse_to_space() {
        let mut out = String::new();
        write_leaf(&mut out, Event::SoftBreak, Escape::Body, false);
        write_leaf(&mut out, Event::HardBreak, Escape::Body, false);
        assert_eq!(out, "  ");
    }

    #[test]
    fn test_write_leaf_escapes_inline_html_for_body() {
        let mut out = String::new();
        write_leaf(
            &mut out,
            Event::InlineHtml("<b>".into()),
            Escape::Body,
            false,
        );
        assert_eq!(out, "&lt;b&gt;");
    }

    #[test]
    fn test_is_formatting_tag() {
        assert!(is_formatting_start(&Tag::Strong));
        assert!(is_formatting_start(&Tag::Strikethrough));
        assert!(!is_formatting_start(&Tag::Paragraph));
        assert!(is_formatting_end(&TagEnd::Emphasis));
        assert!(!is_formatting_end(&TagEnd::Paragraph));
    }
}
