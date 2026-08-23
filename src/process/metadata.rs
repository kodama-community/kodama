// Copyright (c) 2025 Kodama Project. All rights reserved.
// Released under the GPL-3.0 license as described in the file LICENSE.
// Authors: Kokic (@kokic), Spore (@s-cerevisiae)

use crate::{
    compiler::{parser::parse_spanned_markdown, section::HTMLContent},
    entry::{is_plain_metadata, HTMLMetaData, KEY_TAXON, KEY_TITLE},
    slug::Slug,
};
use eyre::eyre;
use pulldown_cmark::{Event, Tag, TagEnd};

pub struct Metadata<'m, E> {
    events: E,
    state: bool,
    metadata: &'m mut HTMLMetaData,
}

impl<'m, E> Metadata<'m, E> {
    pub fn process(events: E, metadata: &'m mut HTMLMetaData) -> Self {
        Self {
            events,
            state: false,
            metadata,
        }
    }
}

impl<'e, E: Iterator<Item = Event<'e>>> Iterator for Metadata<'_, E> {
    type Item = eyre::Result<Event<'e>>;

    fn next(&mut self) -> Option<Self::Item> {
        for e in self.events.by_ref() {
            match e {
                Event::Start(Tag::MetadataBlock(_)) => {
                    self.state = true;
                }
                Event::End(TagEnd::MetadataBlock(_)) => {
                    self.state = false;
                }
                Event::Text(ref text) => {
                    if !self.state || text.trim().is_empty() {
                        return Some(Ok(e));
                    }
                    if let Err(e) = parse_metadata(text, self.metadata) {
                        return Some(Err(e.wrap_err("failed to parse metadata")));
                    }
                }
                _ => return Some(Ok(e)),
            }
        }
        None
    }
}

/// It is known that the behavior differs between the two architectures
/// `(I)` `x86_64-pc-windows-msvc` and `(II)` `aarch64-unknown-linux-musl`.
/// `(I)` automatically splits the input by lines,
/// while `(II)` receives the entire multi-line string as a whole.
fn parse_metadata(s: &str, metadata: &mut HTMLMetaData) -> eyre::Result<()> {
    let current_slug = metadata
        .slug()
        .ok_or_else(|| eyre!("missing `slug` while parsing metadata block"))?;

    for (line_no, s) in s.lines().enumerate() {
        if !s.trim().is_empty() {
            let (key, val) = s.split_once(':').ok_or_else(|| {
                eyre!(
                    "invalid metadata in `{}` at line {}: expected `name: value`, found `{}`",
                    current_slug,
                    line_no + 1,
                    s
                )
            })?;
            let key = key.trim();
            let val = val.trim();

            parse_metadata_value(key, val, current_slug, metadata)?;
        }
    }
    Ok(())
}

fn parse_metadata_value(
    key: &str,
    value: &str,
    current_slug: Slug,
    metadata: &mut HTMLMetaData,
) -> eyre::Result<()> {
    if is_plain_metadata(key) {
        metadata.builtin.assign(key, value, current_slug)?;
        return Ok(());
    }

    let parsed = parse_spanned_markdown(value, current_slug);
    match key {
        KEY_TITLE => metadata.title = Some(parsed),
        KEY_TAXON => {
            metadata.taxon = Some(match parsed {
                HTMLContent::Plain(v) => HTMLContent::Plain(display_taxon(&v)),
                other => other,
            })
        }
        _ => {
            metadata.custom.insert(key.to_string(), parsed);
        }
    }
    Ok(())
}

/// Format the taxon string for display.
pub fn display_taxon(s: &str) -> String {
    // Capitalize the first letter and add a period and space at the end.
    match s.split_at_checked(1) {
        Some((first, rest)) => format!("{}. ", first.to_uppercase() + rest),
        _ => format!("{}. ", s),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metadata_with_slug(slug: &str) -> HTMLMetaData {
        HTMLMetaData::with_slug_ext(Slug::new(slug), "md")
    }

    #[test]
    fn test_page_title_is_plain_text_and_not_elaborated() {
        crate::environment::mock_environment().unwrap();

        let mut metadata = metadata_with_slug("index");
        parse_metadata("page-title: 中文", &mut metadata).unwrap();

        let parsed = metadata.page_title().unwrap_or_default().to_string();
        assert_eq!(parsed, "中文");
        assert!(!parsed.contains("<span"));
    }

    #[test]
    fn test_title_stays_rich_and_allows_text_elaboration() {
        crate::environment::mock_environment().unwrap();

        let mut metadata = metadata_with_slug("index");
        parse_metadata("title: 中文", &mut metadata).unwrap();

        let parsed = metadata
            .title()
            .and_then(HTMLContent::as_str)
            .unwrap_or_default()
            .to_string();
        assert!(parsed.contains("<span lang=\"zh\">"));
    }

    #[test]
    fn test_taxon_keeps_display_formatting() {
        crate::environment::mock_environment().unwrap();

        let mut metadata = metadata_with_slug("index");
        parse_metadata("taxon: remark", &mut metadata).unwrap();

        let parsed = metadata
            .taxon()
            .and_then(HTMLContent::as_str)
            .unwrap_or_default()
            .to_string();
        assert_eq!(parsed, "Remark. ");
    }
}
