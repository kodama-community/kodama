// Copyright (c) 2025 Kodama Project. All rights reserved.
// Released under the GPL-3.0 license as described in the file LICENSE.
// Authors: Kokic (@kokic), Spore (@s-cerevisiae)

use pulldown_cmark::{Event, Tag, TagEnd};

use super::inline_content::{
    is_formatting_end, is_formatting_start, warn_unexpected, write_leaf, Escape,
};

pub struct Figure<E> {
    events: E,
    title: String,
    dest_url: Option<String>,
    nest: usize,
}

impl<E> Figure<E> {
    pub fn process(events: E) -> Self {
        Self {
            events,
            title: String::new(),
            dest_url: None,
            nest: 0,
        }
    }
}

impl<'e, E: Iterator<Item = Event<'e>>> Iterator for Figure<E> {
    type Item = Event<'e>;

    fn next(&mut self) -> Option<Self::Item> {
        for e in self.events.by_ref() {
            match e {
                Event::Start(Tag::Image { dest_url, .. }) => {
                    self.dest_url = Some(dest_url.into());
                    self.title.clear();
                    self.nest = 0;
                }
                Event::End(TagEnd::Image) => {
                    if self.dest_url.is_none() {
                        return Some(e);
                    }
                    if self.nest != 0 {
                        self.nest -= 1;
                        continue;
                    }
                    let dest_url = self.dest_url.take().unwrap_or_default();
                    let title = htmlize::escape_attribute(&self.title);
                    let html = format!(
                        r#"<img src="{}" title="{}" alt="{}">"#,
                        htmlize::escape_attribute(&dest_url),
                        title,
                        title,
                    );
                    self.title.clear();
                    return Some(Event::Html(html.into()));
                }
                Event::Start(ref tag) => {
                    if self.dest_url.is_some() {
                        if is_formatting_start(tag) {
                            self.nest += 1;
                        } else {
                            warn_unexpected("block-level start tag", "image alt text");
                        }
                    } else {
                        return Some(e);
                    }
                }
                Event::End(ref tag) => {
                    if self.dest_url.is_some() {
                        if is_formatting_end(tag) {
                            self.nest = self.nest.saturating_sub(1);
                        } else {
                            warn_unexpected("block-level end tag", "image alt text");
                        }
                    } else {
                        return Some(e);
                    }
                }
                Event::Text(_)
                | Event::Code(_)
                | Event::InlineMath(_)
                | Event::DisplayMath(_)
                | Event::InlineHtml(_)
                | Event::SoftBreak
                | Event::HardBreak => {
                    if self.dest_url.is_some() {
                        write_leaf(&mut self.title, e, Escape::Raw, false);
                    } else {
                        return Some(e);
                    }
                }
                Event::Html(_)
                | Event::FootnoteReference(_)
                | Event::TaskListMarker(_)
                | Event::Rule => {
                    if self.dest_url.is_some() {
                        warn_unexpected("event", "image alt text");
                    } else {
                        return Some(e);
                    }
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn collect<'a>(source: &'a str) -> Vec<Event<'a>> {
        let parser = pulldown_cmark::Parser::new_ext(source, crate::compiler::parser::OPTIONS);
        Figure::process(parser).collect::<Vec<_>>()
    }

    fn img_html(events: &[Event]) -> Option<String> {
        events.iter().find_map(|e| match e {
            Event::Html(html) => Some(html.to_string()),
            _ => None,
        })
    }

    #[test]
    fn test_image_alt_collects_softbreak_math_and_code() {
        let events = collect("![a\nb $x$ `c`](image.png)");
        let html = img_html(&events).expect("expected an img html event");
        assert_eq!(
            html,
            r#"<img src="image.png" title="a b $x$ c" alt="a b $x$ c">"#
        );
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, Event::Html(_)))
                .count(),
            1
        );
    }

    #[test]
    fn test_image_alt_strips_formatting_without_leaking() {
        let events = collect("![*bold* **strong**](image.png)");
        let html = img_html(&events).expect("expected an img html event");
        assert!(html.contains(r#"title="bold strong""#));
        assert!(!events
            .iter()
            .any(|e| matches!(e, Event::Start(_) | Event::End(_) if !matches!(e, Event::Start(Tag::Paragraph) | Event::End(TagEnd::Paragraph)))));
    }

    #[test]
    fn test_image_alt_escapes_raw_html() {
        let events = collect("![a <b> c](image.png)");
        let html = img_html(&events).expect("expected an img html event");
        assert!(html.contains(r#"title="a &lt;b&gt; c""#));
    }
}
