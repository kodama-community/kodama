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

#[cfg(test)]
mod test {
    use camino::{Utf8Path, Utf8PathBuf};
    use std::fs;

    #[test]
    fn test_resolve_config_path_returns_the_given_path() {
        let root = crate::test_io::case_dir("config-resolve-present");
        fs::create_dir_all(root.as_std_path()).unwrap();
        let config_path = root.join("Kodama.toml");
        fs::write(config_path.as_std_path(), "").unwrap();

        let resolved = crate::config::resolve_config_path(config_path.as_path()).unwrap();
        assert_eq!(resolved, config_path);

        let _ = fs::remove_dir_all(root.as_std_path());
    }

    #[test]
    fn test_resolve_config_path_does_not_fall_back_to_the_parent_directory() {
        let root = crate::test_io::case_dir("config-resolve-parent");
        let sub = root.join("sub");
        fs::create_dir_all(sub.as_std_path()).unwrap();
        // A config file exists one level up; it must not be substituted.
        fs::write(root.join("Kodama.toml").as_std_path(), "").unwrap();

        let err = crate::config::resolve_config_path(sub.join("Kodama.toml").as_path())
            .expect_err("missing config must not resolve to the parent directory");
        let message = format!("{err}");
        assert!(
            message.contains("configuration file not found"),
            "{message}"
        );
        assert!(message.contains("sub/Kodama.toml"), "{message}");
        assert!(!message.contains("/./"), "{message}");

        let _ = fs::remove_dir_all(root.as_std_path());
    }

    #[test]
    fn test_missing_config_message_reports_the_current_directory() {
        let err = crate::config::resolve_config_path(Utf8Path::new("./definitely-missing.toml"))
            .expect_err("missing config must be an error");
        let message = format!("{err}");
        let cwd = Utf8PathBuf::from_path_buf(std::env::current_dir().unwrap()).unwrap();
        assert!(message.contains(cwd.as_str()), "{message}");
        assert!(!message.contains("/./"), "{message}");
    }

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
