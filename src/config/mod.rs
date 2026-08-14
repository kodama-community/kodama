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
use camino::{Utf8Path, Utf8PathBuf};
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

/// Try to find toml file in the current directory or the parent directory.
pub fn find_config(toml_file: &Utf8Path) -> eyre::Result<Utf8PathBuf> {
    if !toml_file.exists() {
        let parent = toml_file
            .parent()
            .ok_or_else(|| eyre::eyre!("cannot resolve parent directory of `{}`", toml_file))?
            .canonicalize_utf8()?;
        let parent = parent.parent().ok_or_else(|| {
            eyre::eyre!(
                "cannot find configuration file from root directory while searching from `{}`",
                toml_file
            )
        })?;

        let toml_file = parent.join(DEFAULT_CONFIG_PATH);
        if !toml_file.exists() {
            return Err(eyre::eyre!("cannot find configuration file: {}", toml_file));
        }
        Ok(toml_file)
    } else {
        Ok(toml_file.to_owned())
    }
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
            assert!(crate::config::parse_config(toml).is_err(), "expected error for `{toml}`");
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
}
