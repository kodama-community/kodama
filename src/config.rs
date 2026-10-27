// Copyright (c) 2025 Kodama Project. All rights reserved.
// Released under the GPL-3.0 license as described in the file LICENSE.
// Authors: Kokic (@kokic), Spore (@s-cerevisiae)

pub mod build;
pub mod kodama;
pub mod publish;
pub mod serve;
pub mod suffix;
pub mod text;
pub mod toc;

use build::Build;
use camino::{Utf8Component, Utf8Path, Utf8PathBuf};
use kodama::Kodama;
use publish::Publish;
use serde::{Deserialize, Serialize};
use serve::Serve;
use suffix::Suffix;
use text::Text;
use toc::Toc;

pub const DEFAULT_CONFIG_PATH: &str = "./Kodama.toml";

/// Where a project command takes its configuration from.
///
/// The source decides both the configuration values and the project root, so it
/// is resolved once per command invocation and carried through the build and
/// serve pipelines instead of a bare config path.
#[derive(Debug, Clone)]
pub enum ConfigSource {
    /// Read the configuration file at this path. The path must exist, and its
    /// directory becomes the project root.
    File(Utf8PathBuf),
}

impl ConfigSource {
    pub fn file(path: impl Into<Utf8PathBuf>) -> Self {
        Self::File(path.into())
    }
}

#[derive(Deserialize, Debug, Default, Serialize)]
pub struct Config {
    #[serde(default)]
    pub kodama: Kodama,

    #[serde(default)]
    pub toc: Toc,

    #[serde(default)]
    pub text: Text,

    #[serde(default)]
    pub build: Build,

    #[serde(default)]
    pub serve: Serve,

    #[serde(default)]
    pub publish: Publish,

    #[serde(default)]
    pub suffix: Suffix,
}

/// Resolve the configuration file path used by project commands.
///
/// The given path is authoritative: it is returned as-is when it exists, and a
/// missing path is an error. The path is never substituted with a file from
/// another directory, because the configuration file also determines the
/// project root, and therefore where output is written.
pub fn resolve_config_path(config_path: &Utf8Path) -> eyre::Result<Utf8PathBuf> {
    if !config_path.exists() {
        return Err(eyre::eyre!(
            "configuration file not found: `{}`\n\
             Run from the site root, or pass the configuration file explicitly, for example `-c ../Kodama.toml`.",
            absolute_display(config_path)
        ));
    }
    Ok(config_path.to_owned())
}

/// Render `path` as an absolute path for diagnostics, without changing the
/// path that is actually used. CurDir components (as in the default
/// `./Kodama.toml`) are dropped so messages do not read `site/./Kodama.toml`.
fn absolute_display(path: &Utf8Path) -> String {
    let normalized: Utf8PathBuf = path
        .components()
        .filter(|component| !matches!(component, Utf8Component::CurDir))
        .collect();
    if normalized.is_absolute() {
        return normalized.to_string();
    }
    std::env::current_dir()
        .ok()
        .and_then(|cwd| Utf8PathBuf::from_path_buf(cwd).ok())
        .map(|cwd| cwd.join(&normalized).to_string())
        .unwrap_or_else(|| normalized.to_string())
}

pub fn parse_config(config: &str) -> eyre::Result<Config> {
    let config: Config =
        toml::from_str(config).map_err(|e| eyre::eyre!("failed to parse config file: {}", e))?;
    validate_suffixes(&config.suffix)?;
    Ok(config)
}

/// The suffix values must be pairwise distinct so that a file's role
/// (markdown section, typst section, or typst library) is never ambiguous.
fn validate_suffixes(suffix: &Suffix) -> eyre::Result<()> {
    if suffix.markdown.is_empty() || suffix.typst.is_empty() {
        return Err(eyre::eyre!(
            "[suffix] `markdown` and `typst` must not be empty"
        ));
    }
    if suffix.markdown == suffix.typst
        || suffix.markdown == suffix.typst_lib
        || suffix.typst == suffix.typst_lib
    {
        return Err(eyre::eyre!(
            "[suffix] `markdown`, `typst` and `typst-lib` must be pairwise distinct; got `{}`, `{}`, `{}`",
            suffix.markdown,
            suffix.typst,
            suffix.typst_lib,
        ));
    }
    Ok(())
}

mod test {

    #[test]
    fn test_empty_toml() {
        let serve = crate::config::Serve::default();
        let config = crate::config::parse_config("").unwrap();

        assert_eq!(config.kodama.trees, "trees");
        assert_eq!(config.kodama.assets, "assets");
        assert_eq!(config.kodama.base_url, "/");
        assert!(!config.kodama.theme_lock);
        assert!(!config.build.short_slug);
        assert!(!config.build.pretty_urls);
        assert!(!config.build.inline_css);
        assert!(!config.build.inline_script);
        assert!(!config.build.allow_unsafe_html);
        assert!(config.build.elaborate_cjk_text);
        assert_eq!(config.build.footer_sort_by, "slug");
        assert_eq!(config.serve.edit, serve.edit);
        assert_eq!(config.serve.output, serve.output);
        assert!(!config.publish.rss);
        assert_eq!(config.suffix.markdown, "md");
        assert_eq!(config.suffix.typst, "typst");
        assert_eq!(config.suffix.typst_lib, "typ");
    }

    #[test]
    fn test_simple_toml() {
        let serve = crate::config::Serve::default();
        let config = crate::config::parse_config(
            r#"
            [kodama]
            trees = "source"
            assets = "assets"
            base-url = "https://example.com/"
            theme-lock = true

            [build]
            short-slug = true
            inline-css = true
            inline-script = true
            allow-unsafe-html = true
            footer-sort-by = "title"
            elaborate-cjk-text = false

            [publish]
            rss = true
            "#,
        )
        .unwrap();

        assert_eq!(config.kodama.trees, "source");
        assert_eq!(config.kodama.assets, "assets");
        assert_eq!(config.kodama.base_url, "https://example.com/");
        assert!(config.kodama.theme_lock);
        assert!(config.build.short_slug);
        assert!(config.build.inline_css);
        assert!(config.build.inline_script);
        assert!(config.build.allow_unsafe_html);
        assert!(!config.build.elaborate_cjk_text);
        assert_eq!(config.build.footer_sort_by, "title");
        assert_eq!(config.serve.edit, serve.edit);
        assert_eq!(config.serve.output, serve.output);
        assert!(config.publish.rss);
    }

    #[test]
    fn test_suffix_custom_values() {
        let config = crate::config::parse_config(
            r#"
            [suffix]
            markdown = "markdown"
            typst = "typ"
            typst-lib = "lib"
            "#,
        )
        .unwrap();

        assert_eq!(config.suffix.markdown, "markdown");
        assert_eq!(config.suffix.typst, "typ");
        assert_eq!(config.suffix.typst_lib, "lib");
    }

    #[test]
    fn test_suffix_rejects_colliding_values() {
        let colliding = [
            r#"
            [suffix]
            markdown = "md"
            typst = "md"
            "#,
            r#"
            [suffix]
            markdown = "md"
            typst = "typst"
            typst-lib = "md"
            "#,
            r#"
            [suffix]
            typst = "typ"
            typst-lib = "typ"
            "#,
        ];
        for toml in colliding {
            assert!(
                crate::config::parse_config(toml).is_err(),
                "expected error for `{toml}`"
            );
        }
    }

    #[test]
    fn test_suffix_rejects_empty_markdown_or_typst() {
        let toml = r#"
            [suffix]
            markdown = ""
            typst = "typst"
            "#;
        assert!(crate::config::parse_config(toml).is_err());
    }

    #[test]
    fn test_empty_serve_section_uses_defaults() {
        let config = crate::config::parse_config("[serve]").unwrap();
        assert_eq!(config.serve.output, "./.cache/publish");
        assert!(config.serve.port.is_none());
        assert!(config.serve.command.is_empty());
    }

    #[test]
    fn test_partial_serve_section_defaults_missing_fields() {
        let config = crate::config::parse_config("[serve]\noutput = \"./site\"").unwrap();
        assert_eq!(config.serve.output, "./site");
        assert!(config.serve.port.is_none());
        assert!(config.serve.command.is_empty());
    }

    #[test]
    fn test_serve_port_is_parsed() {
        let config = crate::config::parse_config("[serve]\nport = 4321").unwrap();
        assert_eq!(config.serve.port, Some(4321));
    }

    #[test]
    fn test_serve_port_out_of_range_is_rejected() {
        assert!(crate::config::parse_config("[serve]\nport = 70000").is_err());
    }

    #[test]
    fn test_serve_command_is_parsed_for_external_server() {
        let config = crate::config::parse_config(
            "[serve]\ncommand = [\"miniserve\", \"<output>\", \"--index\", \"index.html\"]",
        )
        .unwrap();
        assert_eq!(
            config.serve.command,
            ["miniserve", "<output>", "--index", "index.html"]
        );
    }
}
