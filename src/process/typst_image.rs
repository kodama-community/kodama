// Copyright (c) 2025 Kodama Project. All rights reserved.
// Released under the GPL-3.0 license as described in the file LICENSE.
// Authors: Kokic (@kokic), Spore (@s-cerevisiae)

use std::{
    cell::RefCell,
    fmt::Write,
    fs,
    sync::atomic::{AtomicBool, Ordering},
};

use camino::Utf8PathBuf;
use pulldown_cmark::{Event, Tag, TagEnd};

use crate::{
    compiler::section::{
        EmbedContent, HTMLContent, LazyContent, LocalLink, UnresolvedSection,
    },
    environment::{self, output_path},
    html_flake::{html_figure_code, html_inline_typst_span, html_typst_figure},
    recorder::State,
    slug::Slug,
    typst_cli::{self, write_to_inline_html},
};

use super::{
    path_resolution::{relocate_trees_path, resolve_section_url},
    processor::url_action,
};

/// Placeholder token emitted at each inline formula's position while the batch
/// compilation is still pending. The token embeds the formula's zero-based
/// index so the results can be substituted back in order afterwards.
const INLINE_PLACEHOLDER: &str = "\u{0}KIL";

/// Inline typst formulas collected during a single parse phase. The whole
/// build is single-threaded, so a thread-local registry lets every parsed
/// section register formulas and defer compilation to the end of the phase,
/// keeping a single `typst c` invocation per build.
#[derive(Default)]
struct InlineBatch {
    /// `(shareds header, formula source, originating slug)` in formula order.
    pending: Vec<(String, String, Slug)>,
}

thread_local! {
    static INLINE_BATCH: RefCell<InlineBatch> = RefCell::new(InlineBatch::default());
}

/// Reset the inline formula registry for a new parse phase.
pub fn begin_inline_batch() {
    INLINE_BATCH.with(|batch| batch.borrow_mut().pending.clear());
}

/// Register an inline formula and return its global (phase-local) index.
fn push_inline_formula(shareds: String, source: String, current_slug: Slug) -> usize {
    INLINE_BATCH.with(|batch| {
        let mut batch = batch.borrow_mut();
        batch.pending.push((shareds, source, current_slug));
        batch.pending.len() - 1
    })
}

/// Compile all formulas collected during the current parse phase. Returns the
/// compiled inline svg html in formula order.
pub fn compile_inline_batch() -> Vec<String> {
    let pending = INLINE_BATCH.with(|batch| std::mem::take(&mut batch.borrow_mut().pending));
    if pending.is_empty() {
        return Vec::new();
    }

    let groups = group_formulas(&pending);
    match typst_cli::source_to_inline_svgs_grouped(&groups) {
        Ok(segments) => wrap_segments(segments),
        Err(_) => {
            // Fall back per group to keep error handling isolated: a failure in
            // one section no longer forces every other section to recompile.
            let mut results = Vec::with_capacity(pending.len());
            let mut idx = 0;
            while idx < pending.len() {
                let header = pending[idx].0.clone();
                let mut end = idx + 1;
                while end < pending.len() && pending[end].0 == header {
                    end += 1;
                }
                let group: Vec<String> = pending[idx..end]
                    .iter()
                    .map(|(_, source, _)| source.clone())
                    .collect();
                match typst_cli::source_to_inline_svgs(&group, &header) {
                    Ok(segments) => results.extend(wrap_segments(segments)),
                    Err(_) => {
                        for (h, source, slug) in &pending[idx..end] {
                            let src =
                                if h.is_empty() { source.clone() } else { format!("{h}\n{source}") };
                            match typst_cli::source_to_inline_svg(&src) {
                                Ok(html) => results.push(html),
                                Err(err) => {
                                    record_typst_image_error();
                                    color_print::ceprintln!(
                                        "<r>{:?} at {}</>",
                                        err,
                                        slug
                                    );
                                    results.push(String::new());
                                }
                            }
                        }
                    }
                }
                idx = end;
            }
            results
        }
    }
}

/// Group consecutive formulas sharing the same shareds header. Consecutive
/// grouping preserves formula order and maps cleanly onto the placeholder
/// indices.
fn group_formulas(pending: &[(String, String, Slug)]) -> Vec<(String, Vec<String>)> {
    let mut groups: Vec<(String, Vec<String>)> = Vec::new();
    for (header, source, _) in pending {
        if groups
            .last()
            .is_some_and(|(last_header, _)| last_header == header)
        {
            groups.last_mut().expect("checked last").1.push(source.clone());
        } else {
            groups.push((header.clone(), vec![source.clone()]));
        }
    }
    groups
}

fn wrap_segments(segments: Vec<String>) -> Vec<String> {
    segments
        .into_iter()
        .map(|segment| format!("\n{}\n", html_inline_typst_span(&segment)))
        .collect()
}

static TYPEST_IMAGE_ERROR_FLAG: AtomicBool = AtomicBool::new(false);

pub fn reset_typst_image_error_flag() {
    TYPEST_IMAGE_ERROR_FLAG.store(false, Ordering::Relaxed);
}

pub fn typst_image_error_detected() -> bool {
    TYPEST_IMAGE_ERROR_FLAG.load(Ordering::Relaxed)
}

fn record_typst_image_error() {
    TYPEST_IMAGE_ERROR_FLAG.store(true, Ordering::Relaxed);
}

pub struct TypstImage<E> {
    events: E,
    state: State,
    shareds: Vec<String>,
    url: Option<String>,
    content: Option<String>,
    current_slug: Slug,
}

impl<E> TypstImage<E> {
    pub fn process(events: E, current_slug: Slug) -> Self {
        Self {
            events,
            state: State::None,
            shareds: Vec::new(),
            url: None,
            content: None,
            current_slug,
        }
    }

    fn exit(&mut self) {
        self.state = State::None;
        self.url = None;
        self.content = None;
    }
}

impl<'e, E: Iterator<Item = Event<'e>>> Iterator for TypstImage<E> {
    type Item = Event<'e>;

    fn next(&mut self) -> Option<Self::Item> {
        for e in self.events.by_ref() {
            match e {
                Event::Start(Tag::Link { ref dest_url, .. }) => {
                    let (url, action) = url_action(dest_url);
                    if is_inline_typst(dest_url) {
                        self.state = State::InlineTypst;
                        self.url = Some(dest_url.to_string()); // [0]
                    } else if action == State::ImageCode.strify() {
                        self.state = State::ImageCode;
                        self.url = Some(url.to_string());
                    } else if action == State::Html.strify() {
                        self.state = State::Html;
                        self.url = Some(url.to_string());
                    } else if action == State::Shared.strify() {
                        self.state = State::Shared;
                        self.url = Some(url.to_string());
                    } else if action == State::ImageBlock.strify() {
                        self.state = State::ImageBlock;
                        self.url = Some(url.to_string());
                    } else if action == State::ImageSpan.strify() {
                        self.state = State::ImageSpan;
                        self.url = Some(url.to_string());
                    } else {
                        return Some(e);
                    }
                }
                Event::Text(ref content) if allow_inline(&self.state) => {
                    self.content.get_or_insert_default().push_str(content);
                }
                Event::InlineMath(ref content) if allow_inline(&self.state) => {
                    let c = self.content.get_or_insert_default();
                    let _ = write!(c, "${content}$");
                }
                Event::Code(ref content) if allow_inline(&self.state) => {
                    let c = self.content.get_or_insert_default();
                    let _ = write!(c, "<code>{content}</code>");
                }
                Event::End(TagEnd::Link) => match self.state {
                    State::Html => {
                        let typst_url =
                            typst_path(self.current_slug, &self.url.take().unwrap_or_default());
                        let html = if environment::is_check() {
                            let trees_dir = environment::trees_dir();
                            match typst_cli::file_to_html(typst_url.as_str(), trees_dir.as_str()) {
                                Ok(inline_html) => inline_html,
                                Err(err) => {
                                    record_typst_image_error();
                                    color_print::ceprintln!(
                                        "<r>{:?} at {}</>",
                                        err,
                                        self.current_slug
                                    );
                                    String::new()
                                }
                            }
                        } else {
                            let html_path = output_path(typst_url.with_extension("html"));
                            match write_to_inline_html(typst_url, html_path) {
                                Ok(inline_html) => inline_html,
                                Err(err) => {
                                    record_typst_image_error();
                                    color_print::ceprintln!(
                                        "<r>{:?} at {}</>",
                                        err,
                                        self.current_slug
                                    );
                                    String::new()
                                }
                            }
                        };

                        self.exit();
                        return Some(Event::Html(html.into()));
                    }
                    State::InlineTypst => {
                        let shareds = self.shareds.join("\n");
                        let inline_url = if let Some(url) = self.url.take() {
                            url
                        } else {
                            color_print::ceprintln!(
                                "<y>Warning: missing inline typst url at `{}`.</>",
                                self.current_slug
                            );
                            self.state = State::None;
                            self.content = None;
                            continue;
                        };
                        let auto_math_mode = inline_url.split('-').skip(1).any(|arg| arg == "math");

                        let mut inline_typst = self.content.take().unwrap_or_default();
                        inline_typst = smart_punctuation_reverse(&inline_typst);

                        if auto_math_mode {
                            inline_typst = format!("${}$", inline_typst);
                        }

                        let placeholder = format!(
                            "{INLINE_PLACEHOLDER}{}\u{0}",
                            push_inline_formula(shareds, inline_typst, self.current_slug)
                        );
                        self.exit();
                        return Some(Event::Html(placeholder.into()));
                    }
                    State::ImageSpan => {
                        let typst_url =
                            typst_path(self.current_slug, &self.url.take().unwrap_or_default());
                        let caption = self.content.take().unwrap_or_default();
                        let svg_url = typst_url.with_extension("svg");
                        self.exit();

                        let html = html_typst_figure(&environment::full_url(&svg_url), false, caption);
                        return Some(Event::Html(html.into()));
                    }
                    State::ImageBlock => {
                        let typst_url =
                            typst_path(self.current_slug, &self.url.take().unwrap_or_default());
                        let caption = self.content.take().unwrap_or_default();
                        let svg_url = typst_url.with_extension("svg");
                        self.exit();

                        let html = html_typst_figure(&environment::full_url(&svg_url), true, caption);
                        return Some(Event::Html(html.into()));
                    }
                    State::ImageCode => {
                        let typst_url =
                            typst_path(self.current_slug, &self.url.take().unwrap_or_default());
                        let caption = self.content.take().unwrap_or_default();
                        let svg_url = typst_url.with_extension("svg");
                        self.exit();

                        let root_dir = environment::trees_dir();
                        let full_path = root_dir.join(typst_url);
                        let code = fs::read_to_string(format!("{}.code", full_path))
                            .or_else(|_| fs::read_to_string(&full_path))
                            .unwrap_or_else(|err| {
                                color_print::ceprintln!(
                                    "<y>Warning: failed to read typst source `{}`: {}</>",
                                    full_path,
                                    err
                                );
                                String::new()
                            });

                        let html =
                            html_figure_code(&environment::full_url(&svg_url), caption, code);
                        return Some(Event::Html(html.into()));
                    }
                    State::Shared => {
                        let Some(typst_url) = self.url.take() else {
                            color_print::ceprintln!(
                                "<y>Warning: missing shared typst url at `{}`.</>",
                                self.current_slug
                            );
                            self.state = State::None;
                            continue;
                        };
                        let imported = self.content.take();
                        /*
                         * Unspecified import items will default to all (*),
                         * but we recommend users to manually enter "*" to avoid ambiguity.
                         */
                        let imported = imported.as_ref().map_or("*", |s| s);
                        self.shareds
                            .push(format!(r#"#import "{typst_url}": {imported}"#));

                        self.state = State::None;
                    }
                    _ => return Some(e),
                },
                _ => return Some(e),
            }
        }

        None
    }
}

/// Replace pending inline formula placeholders with their compiled svg.
fn resolve_inline_placeholder(s: &mut String, results: &[String]) {
    if !s.contains(INLINE_PLACEHOLDER) {
        return;
    }
    let mut out = String::with_capacity(s.len());
    let mut rest = s.as_str();
    while let Some(start) = rest.find(INLINE_PLACEHOLDER) {
        out.push_str(&rest[..start]);
        let after = &rest[start + INLINE_PLACEHOLDER.len()..];
        let Some(close) = after.find('\u{0}') else {
            out.push_str(rest);
            rest = "";
            break;
        };
        let idx = after[..close].parse::<usize>().unwrap_or(0);
        out.push_str(results.get(idx).map(String::as_str).unwrap_or(""));
        rest = &after[close + 1..];
    }
    out.push_str(rest);
    *s = out;
}

fn resolve_inline_typst_content(content: &mut HTMLContent, results: &[String]) {
    match content {
        HTMLContent::Plain(s) => resolve_inline_placeholder(s, results),
        HTMLContent::Lazy(contents) => {
            for content in contents {
                match content {
                    LazyContent::Plain(s) => resolve_inline_placeholder(s, results),
                    LazyContent::Embed(EmbedContent { title, .. }) => {
                        if let Some(title) = title {
                            resolve_inline_placeholder(title, results);
                        }
                    }
                    LazyContent::Local(LocalLink { text, .. }) => {
                        if let Some(text) = text {
                            resolve_inline_placeholder(text, results);
                        }
                    }
                }
            }
        }
    }
}

/// Resolve inline formula placeholders in a parsed section (content and
/// metadata values) using the results of the current phase's batch.
pub fn resolve_inline_typst_section(section: &mut UnresolvedSection, results: &[String]) {
    resolve_inline_typst_content(&mut section.content, results);
    for (_, value) in section.metadata.0.iter_mut() {
        resolve_inline_typst_content(value, results);
    }
}

fn allow_inline(state: &State) -> bool {
    *state == State::Shared
        || *state == State::InlineTypst
        || *state == State::Html
        || *state == State::ImageSpan
        || *state == State::ImageBlock
        || *state == State::ImageCode
}

pub fn is_inline_typst(dest_url: &str) -> bool {
    let key = State::InlineTypst.strify();
    dest_url == key || dest_url.starts_with(&format!("{}-", key))
}

fn typst_path(current_slug: Slug, url: &str) -> Utf8PathBuf {
    let resolved = resolve_section_url(url, current_slug);
    let relocated = relocate_trees_path(&resolved);
    Utf8PathBuf::from(relocated.trim_start_matches('/'))
}

/// Reverses smart punctuation to plain ASCII characters.
fn smart_punctuation_reverse(s: &str) -> String {
    s.replace("“", "\"")
        .replace("”", "\"")
        .replace("‘", "'")
        .replace("’", "'")
        .replace("–", "--")
        .replace("—", "---")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        entry::HTMLMetaData,
        ordered_map::OrderedMap,
        slug::Slug,
    };
    use camino::Utf8PathBuf;

    #[test]
    fn test_typst_path_resolves_relative_paths() {
        crate::environment::mock_environment().unwrap();
        let path = typst_path(Slug::new("guide/chapter/index"), "../fig.typ");
        assert_eq!(path, Utf8PathBuf::from("guide/fig.typ"));
    }

    #[test]
    fn test_typst_path_relocates_trees_absolute_paths() {
        crate::environment::mock_environment().unwrap();
        let path = typst_path(Slug::new("guide/index"), "/trees/ref/plot.typ");
        assert_eq!(path, Utf8PathBuf::from("ref/plot.typ"));
    }

    #[test]
    fn test_typst_path_normalizes_dot_segments() {
        crate::environment::mock_environment().unwrap();
        let path = typst_path(Slug::new("a/b/index"), "./x/../y.typ");
        assert_eq!(path, Utf8PathBuf::from("a/b/y.typ"));
    }

    #[test]
    fn test_group_formulas_batches_consecutive_equal_headers() {
        let slug = Slug::new("a");
        let pending = vec![
            (String::new(), "x".to_string(), slug),
            (String::new(), "y".to_string(), slug),
            ("#import \"lib.typ\": *".to_string(), "z".to_string(), slug),
            ("#import \"lib.typ\": *".to_string(), "w".to_string(), slug),
            (String::new(), "v".to_string(), slug),
        ];
        let groups = group_formulas(&pending);
        assert_eq!(groups.len(), 3);
        assert_eq!(groups[0], (String::new(), vec!["x".to_string(), "y".to_string()]));
        assert_eq!(
            groups[1],
            (
                "#import \"lib.typ\": *".to_string(),
                vec!["z".to_string(), "w".to_string()]
            )
        );
        assert_eq!(groups[2], (String::new(), vec!["v".to_string()]));
    }

    #[test]
    fn test_resolve_inline_typst_section_replaces_placeholders_in_content_and_metadata() {
        let results = vec!["<svg>a</svg>".to_string(), "<svg>b</svg>".to_string()];
        let mut section = UnresolvedSection {
            metadata: HTMLMetaData(OrderedMap::from_iter([
                (
                    "title".to_string(),
                    HTMLContent::Plain(format!("{INLINE_PLACEHOLDER}1\u{0}")),
                ),
            ])),
            content: HTMLContent::Lazy(vec![LazyContent::Plain(format!(
                "x {INLINE_PLACEHOLDER}0\u{0} y"
            ))]),
        };

        resolve_inline_typst_section(&mut section, &results);

        let HTMLContent::Lazy(contents) = &section.content else {
            panic!("expected lazy content");
        };
        let resolved = contents
            .iter()
            .filter_map(|c| match c {
                LazyContent::Plain(s) => Some(s.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(resolved, vec!["x <svg>a</svg> y"]);
        let title = section.metadata.0.get("title").expect("title");
        assert_eq!(title.as_str(), Some("<svg>b</svg>"));
    }

    #[test]
    fn test_resolve_inline_typst_section_ignores_unknown_index() {
        let mut section = UnresolvedSection {
            metadata: HTMLMetaData(OrderedMap::new()),
            content: HTMLContent::Plain(format!(
                "{INLINE_PLACEHOLDER}9\u{0}",
            )),
        };
        resolve_inline_typst_section(&mut section, &["ok".to_string()]);
        assert_eq!(section.content.as_str(), Some(""));
    }
}
