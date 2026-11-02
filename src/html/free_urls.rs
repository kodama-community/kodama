// Copyright (c) 2025 Kodama Project. All rights reserved.
// Released under the GPL-3.0 license as described in the file LICENSE.

use std::sync::LazyLock;

use regex_lite::{Captures, Regex};
use url::Url;

use crate::slug::Slug;

// Keep source-relative URLs independent of the page that eventually embeds them.
// These internal URLs are resolved only when the complete page is rendered.
const ORIGIN: &str = "https://kodama.invalid";
const PREFIX: &str = "kodama-root:";

pub fn split_suffix(url: &str) -> (&str, &str) {
    let at = url.find(['?', '#']).unwrap_or(url.len());
    url.split_at(at)
}

pub fn page(slug: Slug) -> String {
    let mut url = Url::parse(ORIGIN).unwrap();
    url.set_path(&format!("{}.html", slug));
    format!("{PREFIX}{}", &url[url::Position::BeforePath..])
}

pub fn resource(target: &str, source: Slug) -> String {
    if target.is_empty() || target.starts_with(['/', '#', '?']) || Url::parse(target).is_ok() {
        return target.to_string();
    }
    let mut base = Url::parse(ORIGIN).unwrap();
    base.set_path(&format!("{}.md", source));
    match base.join(target) {
        Ok(url) => format!("{PREFIX}{}", &url[url::Position::BeforePath..]),
        Err(_) => target.to_string(),
    }
}

pub fn images<'a>(
    events: impl Iterator<Item = pulldown_cmark::Event<'a>>,
    source: Slug,
) -> impl Iterator<Item = pulldown_cmark::Event<'a>> {
    let free = crate::environment::is_free();
    events.map(move |mut event| {
        if free {
            if let pulldown_cmark::Event::Start(pulldown_cmark::Tag::Image { dest_url, .. }) =
                &mut event
            {
                *dest_url = resource(dest_url, source).into();
            }
        }
        event
    })
}

/// Typst's exported HTML also needs to retain the origin of relative resources.
pub fn resources_in_html(html: &str, source: Slug) -> String {
    static ATTRIBUTES: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r#"\b(href|src)="([^"]*)""#).unwrap());
    ATTRIBUTES
        .replace_all(html, |capture: &Captures<'_>| {
            let target = htmlize::unescape_attribute(&capture[2]);
            let target = resource(&target, source);
            format!(
                r#"{}="{}""#,
                &capture[1],
                htmlize::escape_attribute(&target)
            )
        })
        .into_owned()
}

pub fn render_content(content: &mut crate::compiler::section::HTMLContent, page: Slug) {
    use crate::compiler::section::{HTMLContent, LazyContent};
    match content {
        HTMLContent::Plain(html) => *html = render(html, page),
        HTMLContent::Lazy(contents) => {
            for content in contents {
                let html = match content {
                    LazyContent::Plain(html) => Some(html),
                    LazyContent::Embed(embed) => embed.title.as_mut(),
                    LazyContent::Local(link) => link.text.as_mut(),
                    LazyContent::TypstFigure(figure) => Some(&mut figure.caption),
                };
                if let Some(html) = html {
                    *html = render(html, page);
                }
            }
        }
    }
}

pub fn render(html: &str, page: Slug) -> String {
    static URL_ATTRIBUTES: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r#"\b(href|src)="kodama-root:([^"]*)""#).unwrap());
    let mut base = Url::parse(ORIGIN).unwrap();
    base.set_path(&format!("{}.html", page));
    URL_ATTRIBUTES
        .replace_all(html, |capture: &Captures<'_>| {
            let target = htmlize::unescape_attribute(&capture[2]);
            let Ok(url) = base.join(&target) else {
                return capture[0].to_string();
            };
            let relative = base.make_relative(&url).unwrap();
            let relative = if relative.is_empty() {
                base.path().rsplit('/').next().unwrap().to_string()
            } else {
                relative
            };
            format!(
                r#"{}="{}""#,
                &capture[1],
                htmlize::escape_attribute(&relative)
            )
        })
        .into_owned()
}
