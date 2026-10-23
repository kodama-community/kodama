// Copyright (c) 2025 Kodama Project. All rights reserved.
// Released under the GPL-3.0 license as described in the file LICENSE.
// Authors: Kokic (@kokic), Spore (@s-cerevisiae)

use std::{fs, io::Write, process::Command, sync::LazyLock};

use camino::Utf8Path;
use eyre::{eyre, WrapErr};
use regex_lite::Regex;

use crate::{
    environment, html_flake, path_utils,
    typst_cache::{self, Dependencies, DependencyFile},
};

pub fn write_to_inline_html<P: AsRef<Utf8Path>>(
    typst_path: P,
    html_path: P,
) -> eyre::Result<String> {
    let root_dir = environment::trees_dir();
    if typst_cache::artifact_is_current(typst_path.as_ref(), html_path.as_ref(), "html")? {
        let existed_html = fs::read_to_string(html_path.as_ref())?;
        if let Ok(existed_html) = html_to_body_content(&existed_html) {
            if *crate::cli::build::verbose_skip() {
                println!("Skip: {}", path_utils::pretty_path(typst_path.as_ref()));
            }
            return Ok(existed_html);
        }
        color_print::ceprintln!(
            "<y>Warning: cached HTML `{}` is malformed, recompiling.</>",
            path_utils::pretty_path(html_path.as_ref())
        );
    }

    let (html, dependencies) = to_html_string(typst_path.as_ref(), &root_dir)?;
    let html_body = html_to_body_content(&html)?;

    fs::write(html_path.as_ref(), html)?;
    typst_cache::save_artifact(
        typst_path.as_ref(),
        html_path.as_ref(),
        "html",
        dependencies,
    )?;
    if *crate::cli::build::verbose() {
        println!(
            "Compiled to HTML: {}",
            path_utils::pretty_path(html_path.as_ref())
        );
    }
    Ok(html_body)
}

pub fn html_to_body_content(html: &str) -> eyre::Result<String> {
    let root = element_content(html, "html")
        .ok_or_else(|| eyre!("missing `<html>` tags in typst html output"))?;
    let body = extract_body_content(root)
        .ok_or_else(|| eyre!("missing `<body>` tags in typst html output"))?;
    let mut content = String::new();
    if let Some(head) = element_content(root, "head") {
        for style in style_elements().find_iter(head) {
            // Keep styles with the cached fragment. The document writer moves
            // these marked elements into the page head and deduplicates them.
            content.push_str("<style data-kodama-typst");
            content.push_str(&style.as_str()["<style".len()..]);
        }
    }
    content.push_str(body);
    Ok(content)
}

fn style_elements() -> &'static Regex {
    static STYLES: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?s)<style(?:\s[^>]*|)>.*?</style\s*>").unwrap());
    &STYLES
}

pub(crate) fn take_typst_styles(html: &str, styles: &mut Vec<String>) -> String {
    static MARKED_STYLES: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?s)<style data-kodama-typst(?:\s[^>]*|)>.*?</style\s*>").unwrap()
    });
    MARKED_STYLES
        .replace_all(html, |capture: &regex_lite::Captures<'_>| {
            let style = capture[0].replacen(" data-kodama-typst", "", 1);
            if !styles.contains(&style) {
                styles.push(style);
            }
            String::new()
        })
        .into_owned()
}

/// An owned delimiter, independent of Typst's generated paragraph structure.
pub const INLINE_SEGMENT_TAG: &str = "kodama-inline-segment";

pub fn source_to_inline_svg(src: &str) -> eyre::Result<String> {
    let content = source_to_inline_svgs(&[src.to_string()], "")?
        .pop()
        .ok_or_else(|| eyre!("missing inline Typst segment"))?;
    Ok(format!(
        "\n{}\n",
        html_flake::html_inline_typst_span(&content)
    ))
}

/// Compile several inline formulas in a single typst invocation.
///
/// Each formula is wrapped in an owned element so its output can be split
/// back out afterwards. On failure, the caller is expected to fall back to
/// [`source_to_inline_svg`] per formula for precise error reporting.
pub fn source_to_inline_svgs(sources: &[String], shareds: &str) -> eyre::Result<Vec<String>> {
    let kodama_header = include_str!("include/kodama.typ");
    let mut input = format!("{}\n#show: kodama\n{}\n", kodama_header, shareds);
    for source in sources {
        input.push_str(&format!(
            "#html.elem(\"{INLINE_SEGMENT_TAG}\", [{source}])\n"
        ));
    }

    let svg = source_to_html(input.as_str())?;
    let segments = split_inline_segments(&svg)?;
    if segments.len() != sources.len() {
        return Err(eyre!(
            "internal inline typst batching error: expected {} segments, got {}",
            sources.len(),
            segments.len()
        ));
    }
    Ok(segments)
}

/// Compile several groups of inline formulas in a single typst invocation.
///
/// Each group carries its own `shareds` header (import lines). Groups are
/// wrapped in distinct scoped functions so their imports never leak into one
/// another, and every formula is delimited by an owned element so the
/// output can be split back out in index order. On failure the caller falls
/// back to [`source_to_inline_svgs`] per group.
pub fn source_to_inline_svgs_grouped(
    groups: &[(String, Vec<String>)],
) -> eyre::Result<Vec<String>> {
    let kodama_header = include_str!("include/kodama.typ");
    let mut input = format!("{}\n#show: kodama\n", kodama_header);
    for (group_idx, (shareds, sources)) in groups.iter().enumerate() {
        input.push_str(&format!("#let _segment_group_{group_idx}() = {{\n"));
        if !shareds.trim().is_empty() {
            for line in shareds.lines() {
                input.push_str(line.strip_prefix('#').unwrap_or(line));
                input.push('\n');
            }
        }
        for source in sources {
            input.push_str(&format!(
                "  html.elem(\"{INLINE_SEGMENT_TAG}\", [{source}])\n"
            ));
        }
        input.push_str("}\n");
        input.push_str(&format!("#_segment_group_{group_idx}()\n"));
    }

    let svg = source_to_html(input.as_str())?;
    let segments = split_inline_segments(&svg)?;
    let expected: usize = groups.iter().map(|(_, sources)| sources.len()).sum();
    if segments.len() != expected {
        return Err(eyre!(
            "internal inline typst batching error: expected {} segments, got {}",
            expected,
            segments.len()
        ));
    }
    Ok(segments)
}

fn extract_body_content(html: &str) -> Option<&str> {
    // A script or stylesheet in the head can contain a literal `<body>`.
    let html = html
        .find("</head>")
        .map_or(html, |end| &html[end + "</head>".len()..]);
    element_content(html, "body")
}

fn element_content<'a>(html: &'a str, name: &str) -> Option<&'a str> {
    let opening = Regex::new(&format!(
        r#"<{name}(?:\s+(?:[^"'<>]|"[^"]*"|'[^']*')*)?\s*>"#
    ))
    .ok()?;
    let start = opening.find(html)?.end();
    let end = html.rfind(&format!("</{name}>"))?;
    (end >= start).then(|| &html[start..end])
}

fn split_inline_segments(html: &str) -> eyre::Result<Vec<String>> {
    let body = extract_body_content(html)
        .ok_or_else(|| eyre!("missing `<body>` tags in typst inline svg output"))?;
    static TAGS: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(&format!(
            r#"<(?<close>/)?{INLINE_SEGMENT_TAG}(?:\s+(?:[^"'<>]|"[^"]*"|'[^']*')*)?\s*>"#
        ))
        .unwrap()
    });
    let mut segments = Vec::new();
    let mut depth = 0;
    let mut start = 0;
    for capture in TAGS.captures_iter(body) {
        let tag = capture.get(0).unwrap();
        if capture.name("close").is_some() {
            if depth == 0 {
                return Err(eyre!("unexpected closing inline Typst segment"));
            }
            depth -= 1;
            if depth == 0 {
                segments.push(body[start..tag.start()].to_string());
            }
        } else {
            if depth == 0 {
                start = tag.end();
            }
            depth += 1;
        }
    }
    if depth != 0 {
        return Err(eyre!("unclosed inline Typst segment"));
    }
    Ok(segments)
}

pub fn file_to_html(rel_path: &str, root_dir: &str) -> eyre::Result<String> {
    to_html_string(rel_path, root_dir)
        .and_then(|(html, _)| html_to_body_content(&html))
        .wrap_err_with(|| eyre!("failed to extract html body from `{}`", rel_path))
}

fn to_html_string<P: AsRef<Utf8Path>>(
    rel_path: P,
    root_dir: P,
) -> eyre::Result<(String, Dependencies)> {
    let root_dir = root_dir.as_ref();
    let rel_path = rel_path.as_ref();
    let full_path = root_dir.join(rel_path);
    if !full_path.exists() {
        return Err(eyre!(
            "typst source `{}` not found at `{}` (typst root: `{}`). Check `[build].typst-root` in \"Kodama.toml\"",
            rel_path,
            full_path,
            root_dir
        ));
    }

    let mut command = Command::new("typst");
    command
        .arg("c")
        .arg("-f=html")
        .arg("--features=html")
        .arg(format!("--root={}", root_dir))
        .args(["--input", &format!("path={}", rel_path)])
        .args(["--input", &format!("random={}", fastrand::i64(0..))])
        .arg(full_path)
        .arg("-")
        .stdout(std::process::Stdio::piped());
    let dependencies = DependencyFile::new(&mut command, false)?;
    let output = command.output()?;

    if output.status.success() {
        Ok((String::from_utf8(output.stdout)?, dependencies.read()?))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        failed_in_file(concat!(file!(), '#', line!()), rel_path.as_str(), stderr);
        Err(eyre!(
            "typst html compilation failed for `{}`",
            rel_path.as_str()
        ))
    }
}

fn source_to_html(src: &str) -> eyre::Result<String> {
    let root_dir = environment::trees_dir();

    let mut command = Command::new("typst");
    command
        .arg("c")
        .arg("-f=html")
        .arg("--features=html")
        .arg(format!("--root={}", root_dir))
        .arg("-")
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let dependencies = DependencyFile::new(&mut command, true)?;
    let mut typst = command.spawn()?;

    {
        let mut stdin = typst
            .stdin
            .take()
            .ok_or_else(|| eyre!("failed to open stdin for typst process"))?;
        stdin
            .write_all(src.as_bytes())
            .wrap_err("failed to write typst source to stdin")?;
    }

    let output = typst.wait_with_output()?;
    if output.status.success() {
        dependencies.read()?;
        Ok(String::from_utf8(output.stdout)?)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        color_print::ceprintln!(
            "<r>Command failed in {}: \n  {}</>",
            concat!(file!(), '#', line!()),
            stderr
        );
        Err(eyre!("typst inline html compilation failed"))
    }
}

pub fn write_svg<P: AsRef<Utf8Path>>(typst_path: P, svg_path: P) -> eyre::Result<()> {
    let typst_path = typst_path.as_ref();
    let svg_path = svg_path.as_ref();
    let trees_dir = environment::trees_dir();
    let full_path = trees_dir.join(typst_path);

    if typst_cache::artifact_is_current(typst_path, svg_path, "svg")? {
        if *crate::cli::build::verbose_skip() {
            println!("Skip: {}", path_utils::pretty_path(typst_path));
        }
        return Ok(());
    }

    let mut command = Command::new("typst");
    command
        .arg("c")
        .arg("-f=svg")
        .arg(format!("--root={}", trees_dir))
        .arg(&full_path)
        .arg(svg_path);
    let dependencies = DependencyFile::new(&mut command, false)?;
    let output = command.output()?;

    if output.status.success() {
        typst_cache::save_artifact(typst_path, svg_path, "svg", dependencies.read()?)?;
        if *crate::cli::build::verbose() {
            println!(
                "Compiled to SVG: {}",
                path_utils::pretty_path(Utf8Path::new(svg_path))
            );
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        failed_in_file(concat!(file!(), '#', line!()), full_path.as_str(), stderr);
        return Err(eyre!("typst SVG compilation failed for `{}`", typst_path));
    }
    Ok(())
}

fn failed_in_file(src_pos: &'static str, file_path: &str, stderr: std::borrow::Cow<'_, str>) {
    color_print::ceprintln!(
        "<r>Command failed in {}: \n  In file {}, {}</>",
        src_pos,
        file_path,
        stderr
    );
}

#[cfg(test)]
mod tests {
    use super::{html_to_body_content, to_html_string};
    use camino::Utf8Path;
    use std::fs;

    #[test]
    fn test_html_to_body_content_ok() {
        let html = "<html><head></head><body><p>Hello</p></body></html>";
        let body = html_to_body_content(html).unwrap();
        assert_eq!(body, "<p>Hello</p>");
    }

    #[test]
    fn test_html_to_body_content_with_html_attributes() {
        let html =
            r#"<html lang="en"><head></head><body data-note="a > b"><p>Hello</p></body></html>"#;
        let body = html_to_body_content(html).unwrap();
        assert_eq!(body, "<p>Hello</p>");
    }

    #[test]
    fn test_html_to_body_content_missing_tags() {
        let err = html_to_body_content("<p>Hello</p>");
        assert!(err.is_err());
        assert!(html_to_body_content("<html><head></head><p>Hello</p></html>").is_err());
    }

    #[test]
    fn test_html_to_body_content_preserves_styles_without_document_tags() {
        let html = "<html lang='en'><head><meta charset='utf-8'><style>.math { color: red }</style></head><body><p>Formula</p></body></html>";
        let fragment = html_to_body_content(html).unwrap();
        let mut styles = Vec::new();
        let body = super::take_typst_styles(&fragment.repeat(2), &mut styles);
        assert_eq!(body, "<p>Formula</p><p>Formula</p>");
        assert_eq!(styles, ["<style>.math { color: red }</style>"]);
    }

    #[test]
    fn test_body_extraction_ignores_body_text_in_head() {
        let html = r#"<html><head><script>const tag = '<body>';</script></head><body>Real body</body></html>"#;
        assert_eq!(html_to_body_content(html).unwrap(), "Real body");
    }

    #[test]
    fn test_split_inline_segments_returns_each_segment_body() {
        let html = concat!(
            "<html><head><style>.x{}</style></head><body>",
            "<kodama-inline-segment><math>a</math></kodama-inline-segment>",
            "<kodama-inline-segment><svg>c</svg></kodama-inline-segment>",
            "</body></html>",
        );
        let segments = super::split_inline_segments(html).unwrap();
        assert_eq!(segments, vec!["<math>a</math>", "<svg>c</svg>"]);
    }

    #[test]
    fn test_split_inline_segments_ignores_head_content() {
        let html = "<html><head><kodama-inline-segment>fake</kodama-inline-segment></head><body></body></html>";
        let segments = super::split_inline_segments(html).unwrap();
        assert!(segments.is_empty());
    }

    #[test]
    fn test_split_inline_segments_missing_body_is_error() {
        assert!(super::split_inline_segments("<html><p>x</p></html>").is_err());
    }

    #[test]
    fn test_split_inline_segments_unclosed_segment_is_error() {
        let html = "<html><body><kodama-inline-segment>x</body></html>";
        assert!(super::split_inline_segments(html).is_err());
    }

    #[test]
    fn test_inline_segments_accept_attributes_and_preserve_nested_content() {
        let content = "<span>Before</span><kodama-inline-segment>Nested</kodama-inline-segment><span>After</span>";
        let html = format!("<html><body><kodama-inline-segment data-note='a > b'>{content}</kodama-inline-segment></body></html>");
        assert_eq!(super::split_inline_segments(&html).unwrap(), [content]);
    }

    #[test]
    fn test_to_html_string_reports_typst_root_hint_for_missing_source() {
        let base = crate::test_io::case_dir("typst-root-hint");
        fs::create_dir_all(&base).unwrap();

        let err = to_html_string(
            Utf8Path::new("references/feature-typst.typst"),
            base.as_path(),
        )
        .unwrap_err();
        let message = format!("{err:#}");
        assert!(message.contains("Check `[build].typst-root`"));
        assert!(message.contains("references/feature-typst.typst"));

        let _ = fs::remove_dir_all(base.as_std_path());
    }
}
