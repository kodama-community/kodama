// Copyright (c) 2025 Kodama Project. All rights reserved.
// Released under the GPL-3.0 license as described in the file LICENSE.
// Authors: Alias Qli (@AliasQli)

use eyre::eyre;
use htmlize::unescape_attribute;
use regex_lite::{CaptureMatches, Captures, Regex};
use std::borrow::Cow;
use std::collections::HashMap;
use std::str;
use std::sync::LazyLock;

#[derive(Clone, Copy)]
pub enum HTMLTagKind {
    Meta,
    Embed,
    Subtree,
    Local { span: bool },
}

impl HTMLTagKind {
    fn new(name: &str, span: bool) -> Option<HTMLTagKind> {
        match name {
            "meta" => Some(HTMLTagKind::Meta),
            "embed" => Some(HTMLTagKind::Embed),
            "subtree" => Some(HTMLTagKind::Subtree),
            "local" => Some(HTMLTagKind::Local { span }),
            _ => None,
        }
    }

    fn tri_equal(&self, k: &HTMLTagKind) -> Option<bool> {
        match (self, k) {
            (HTMLTagKind::Meta, HTMLTagKind::Meta)
            | (HTMLTagKind::Embed, HTMLTagKind::Embed)
            | (HTMLTagKind::Subtree, HTMLTagKind::Subtree) => Some(true),
            (HTMLTagKind::Local { span: a }, HTMLTagKind::Local { span: b }) => {
                if a == b {
                    Some(true)
                } else {
                    None
                }
            }
            _ => Some(false),
        }
    }
}

struct HTMLTag {
    kind: HTMLTagKind,
    start: usize,
    end: usize,
    mid: Option<usize>,
}

pub struct HTMLMatch<'a> {
    pub kind: HTMLTagKind,
    pub start: usize,
    pub end: usize,
    pub attrs: HashMap<&'a str, Cow<'a, str>>,
    pub body: &'a str,
}

pub struct HTMLParser<'a> {
    html_str: &'a str,
    captures: CaptureMatches<'static, 'a>,
}

impl<'a> HTMLParser<'a> {
    pub fn new(html_str: &'a str) -> HTMLParser<'a> {
        static RE_TAG: LazyLock<Regex> = LazyLock::new(|| {
            fn real(alt: u8) -> String {
                format!(r#"?<real{}>"#, alt)
            }
            fn kodama(alt: u8) -> String {
                format!(r#"kodama-(?<tag{}>meta|embed|subtree|local)"#, alt)
            }
            fn local(alt: u8) -> String {
                format!(r#"kodama-(?<tag{}>local)"#, alt)
            }
            fn attrs(alt: u8) -> String {
                format!(
                    r#"(?<attrs{}>(\s+[a-zA-Z_:][a-zA-Z0-9_.:-]*(\s*=\s*("[^"]*"|'[^']*'|[^\s"'=<>`]+))?)*)\s*"#,
                    alt
                )
            }
            Regex::new(&format!(
                r#"<span>\s*({}<{}{}>)|({}</{}>)\s*</span>|<{}{}>|</{}>"#,
                real(0),
                local(0),
                attrs(0),
                real(1),
                local(1),
                kodama(2),
                attrs(2),
                kodama(3),
            ))
            .unwrap()
        });
        HTMLParser {
            html_str,
            captures: RE_TAG.captures_iter(html_str),
        }
    }
}

impl<'a> Iterator for HTMLParser<'a> {
    type Item = eyre::Result<HTMLMatch<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        fn get_tag(capture: Captures<'_>) -> eyre::Result<(HTMLTag, Option<&str>)> {
            let all = capture
                .get(0)
                .ok_or_else(|| eyre!("missing full match while parsing typst html tag"))?;
            let make_tag = |kind, mid| HTMLTag {
                start: all.start(),
                end: all.end(),
                mid,
                kind,
            };
            if let Some(name) = capture.name("tag0") {
                Ok((
                    make_tag(
                        HTMLTagKind::new(name.as_str(), true)
                            .ok_or_else(|| eyre!("unknown kodama tag `{}`", name.as_str()))?,
                        Some(
                            capture
                                .name("real0")
                                .ok_or_else(|| eyre!("missing `real0` capture"))?
                                .start(),
                        ),
                    ),
                    Some(
                        capture
                            .name("attrs0")
                            .ok_or_else(|| eyre!("missing `attrs0` capture"))?
                            .as_str(),
                    ),
                ))
            } else if let Some(name) = capture.name("tag1") {
                Ok((
                    make_tag(
                        HTMLTagKind::new(name.as_str(), true)
                            .ok_or_else(|| eyre!("unknown kodama tag `{}`", name.as_str()))?,
                        Some(
                            capture
                                .name("real1")
                                .ok_or_else(|| eyre!("missing `real1` capture"))?
                                .end(),
                        ),
                    ),
                    None,
                ))
            } else if let Some(name) = capture.name("tag2") {
                Ok((
                    make_tag(
                        HTMLTagKind::new(name.as_str(), false)
                            .ok_or_else(|| eyre!("unknown kodama tag `{}`", name.as_str()))?,
                        None,
                    ),
                    Some(
                        capture
                            .name("attrs2")
                            .ok_or_else(|| eyre!("missing `attrs2` capture"))?
                            .as_str(),
                    ),
                ))
            } else if let Some(name) = capture.name("tag3") {
                Ok((
                    make_tag(
                        HTMLTagKind::new(name.as_str(), false)
                            .ok_or_else(|| eyre!("unknown kodama tag `{}`", name.as_str()))?,
                        None,
                    ),
                    None,
                ))
            } else {
                Err(eyre!("unexpected tag capture while parsing typst html"))
            }
        }

        let mut stack = vec![];

        let capture = self.captures.next()?;
        let (mut open_tag, mattrs) = match get_tag(capture) {
            Ok(tag) => tag,
            Err(err) => return Some(Err(err)),
        };
        let Some(attrs_str) = mattrs else {
            return Some(Err(eyre!("expecting open kodama tag, found closing tag")));
        };
        stack.push(open_tag.kind);

        let mut close_tag = loop {
            let Some(capture) = self.captures.next() else {
                return Some(Err(eyre!("unclosed kodama tag: unexpected end of html")));
            };
            let (tag, mattrs) = match get_tag(capture) {
                Ok(tag) => tag,
                Err(err) => return Some(Err(err)),
            };

            if mattrs.is_some() {
                stack.push(tag.kind);
            } else {
                let Some(last) = stack.pop() else {
                    return Some(Err(eyre!("found closing tag without matching open tag")));
                };
                if tag.kind.tri_equal(&last) == Some(false) {
                    return Some(Err(eyre!("mismatched nested kodama tags")));
                }
                if stack.is_empty() {
                    break tag;
                }
            }
        };

        if open_tag.kind.tri_equal(&close_tag.kind) != Some(true) {
            open_tag.mid.inspect(|mid| open_tag.start = *mid);
            close_tag.mid.inspect(|mid| close_tag.end = *mid);
        }

        static RE_ATTR: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(
                r#"(?<key>[a-zA-Z_:][a-zA-Z0-9_.:-]*)(\s*=\s*("(?<double>[^"]*)"|'(?<single>[^']*)'|(?<unquoted>[^\s"'=<>`]+)))?"#,
            )
            .unwrap()
        });

        let mut attrs: HashMap<&str, Cow<'_, str>> = HashMap::new();
        for c in RE_ATTR.captures_iter(attrs_str) {
            let Some(key) = c.name("key") else {
                return Some(Err(eyre!("malformed attribute in typst html tag")));
            };
            attrs.insert(
                key.as_str(),
                unescape_attribute(
                    c.name("double")
                        .or_else(|| c.name("single"))
                        .or_else(|| c.name("unquoted"))
                        .map_or("", |m| m.as_str()),
                ),
            );
        }

        Some(Ok(HTMLMatch {
            kind: open_tag.kind,
            start: open_tag.start,
            end: close_tag.end,
            attrs,
            body: self.html_str[open_tag.end..close_tag.start].trim(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::HTMLParser;

    #[test]
    fn attributes_preserve_literal_backslashes() {
        let html = r#"<kodama-meta key="title" value="C:\notes\"></kodama-meta>"#;
        let tag = HTMLParser::new(html).next().unwrap().unwrap();
        assert_eq!(tag.attrs["value"], "C:\\notes\\");
    }

    #[test]
    fn attributes_accept_html_quoting_and_entities() {
        for html in [
            r#"<kodama-meta key="title" value="A &quot;quote&quot; &amp; B"></kodama-meta>"#,
            r#"<kodama-meta key='title' value='A &quot;quote&quot; &amp; B'></kodama-meta>"#,
            r#"<kodama-meta key=title value=A&#32;&quot;quote&quot;&#32;&amp;&#32;B ></kodama-meta>"#,
        ] {
            let tag = HTMLParser::new(html).next().unwrap().unwrap();
            assert_eq!(tag.attrs["key"], "title");
            assert_eq!(tag.attrs["value"], "A \"quote\" & B");
        }
    }

    #[test]
    fn attributes_do_not_confuse_nested_marker_boundaries() {
        let html = r#"<kodama-subtree slug='child' data-v1="x > y"><kodama-meta key=title value='Child'></kodama-meta></kodama-subtree>"#;
        let tag = HTMLParser::new(html).next().unwrap().unwrap();
        assert_eq!(tag.attrs["slug"], "child");
        assert_eq!(tag.attrs["data-v1"], "x > y");
        let nested = HTMLParser::new(tag.body).next().unwrap().unwrap();
        assert_eq!(nested.attrs["value"], "Child");
    }
}
