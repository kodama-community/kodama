// Copyright (c) 2025 Kodama Project. All rights reserved.
// Released under the GPL-3.0 license as described in the file LICENSE.
// Authors: Kokic (@kokic), Spore (@s-cerevisiae)

use std::{
    cell::RefCell,
    sync::atomic::{AtomicBool, Ordering},
};

use crate::{
    compiler::section::{EmbedContent, HTMLContent, LazyContent, LocalLink, UnresolvedSection},
    html_flake::html_inline_typst_span,
    slug::Slug,
    typst_cli,
};

/// Placeholder token emitted at each inline formula's position while the batch
/// compilation is still pending. The token embeds the formula's zero-based
/// index so the results can be substituted back in order afterwards.
pub(crate) const INLINE_PLACEHOLDER: &str = "\u{0}KIL";

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
pub(crate) fn push_inline_formula(shareds: String, source: String, current_slug: Slug) -> usize {
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
                            let src = if h.is_empty() {
                                source.clone()
                            } else {
                                format!("{h}\n{source}")
                            };
                            match typst_cli::source_to_inline_svg(&src) {
                                Ok(html) => results.push(html),
                                Err(err) => {
                                    record_typst_render_error();
                                    color_print::ceprintln!("<r>{:?} at {}</>", err, slug);
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
            groups
                .last_mut()
                .expect("checked last")
                .1
                .push(source.clone());
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

/// Whether any typst compilation failed while elaborating markdown content.
/// Used by the `check` command to fail when render errors are detected.
static TYPST_RENDER_ERROR_FLAG: AtomicBool = AtomicBool::new(false);

pub fn reset_typst_render_error_flag() {
    TYPST_RENDER_ERROR_FLAG.store(false, Ordering::Relaxed);
}

pub fn typst_render_error_detected() -> bool {
    TYPST_RENDER_ERROR_FLAG.load(Ordering::Relaxed)
}

pub(crate) fn record_typst_render_error() {
    TYPST_RENDER_ERROR_FLAG.store(true, Ordering::Relaxed);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        compiler::section::{LazyContent, UnresolvedSection},
        entry::HTMLMetaData,
        ordered_map::OrderedMap,
    };

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
        assert_eq!(
            groups[0],
            (String::new(), vec!["x".to_string(), "y".to_string()])
        );
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
            metadata: HTMLMetaData(OrderedMap::from_iter([(
                "title".to_string(),
                HTMLContent::Plain(format!("{INLINE_PLACEHOLDER}1\u{0}")),
            )])),
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
            content: HTMLContent::Plain(format!("{INLINE_PLACEHOLDER}9\u{0}",)),
        };
        resolve_inline_typst_section(&mut section, &["ok".to_string()]);
        assert_eq!(section.content.as_str(), Some(""));
    }
}
