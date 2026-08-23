// Copyright (c) 2025 Kodama Project. All rights reserved.
// Released under the GPL-3.0 license as described in the file LICENSE.
// Authors: Kokic (@kokic), Spore (@s-cerevisiae)

use crate::{
    compiler::{section::HTMLContent, taxon::Taxon},
    config::build::FooterMode,
    environment, html_flake,
    ordered_map::OrderedMap,
    slug::Slug,
};
use eyre::eyre;
use serde::{Deserialize, Serialize};

pub const KEY_TITLE: &str = "title";

/// Auto-detected
pub const KEY_SLUG: &str = "slug";

/// Auto-detected
pub const KEY_EXT: &str = "ext";

pub const KEY_TAXON: &str = "taxon";
pub const KEY_DATA_TAXON: &str = "data-taxon";

/// Control the "Previous Level" information in the current page navigation.
pub const KEY_PARENT: &str = "parent";

/// Control the page title text of the current page.
pub const KEY_PAGE_TITLE: &str = "page-title";
pub const KEY_SOURCE_SLUG: &str = "source-slug";
pub const KEY_SOURCE_POS: &str = "source-pos";
pub const KEY_INTERNAL_ANON_SUBTREE: &str = "internal-anon-subtree";

/// `backlinks: bool`:
/// Controls whether the current page displays backlinks.
pub const KEY_BACKLINKS: &str = "backlinks";

/// `transparent-backlinks: bool`:
/// Controls whether backlinks of current section is always displayed,
/// even when embedded (except in footer).
/// Default is `false`.
pub const KEY_TRANSPARENT_BACKLINKS: &str = "transparent-backlinks";

/// `references: bool`:
/// Controls whether the current page displays references.
pub const KEY_REFERENCES: &str = "references";

/// `collect: bool`:
/// Controls whether the current page is a collection page.
/// A collection page displays metadata of child entries.
pub const KEY_COLLECT: &str = "collect";

/// `asref: bool`:
/// Controls whether the current page process as reference.
/// Default is `false`.
pub const KEY_ASREF: &str = "asref";

/// `asback: bool`:
/// Controls whether the current page process as backlink.
/// Default is `true`.
pub const KEY_ASBACK: &str = "asback";

/// `footer-mode: embed | link`
pub const KEY_FOOTER_MODE: &str = "footer-mode";

/// `footer-sort-by: <metadata-key>`
pub const KEY_FOOTER_SORT_BY: &str = "footer-sort-by";

const PLAIN_METADATA: [&str; 16] = [
    KEY_SLUG,
    KEY_EXT,
    KEY_DATA_TAXON,
    KEY_PARENT,
    KEY_PAGE_TITLE,
    KEY_SOURCE_SLUG,
    KEY_SOURCE_POS,
    KEY_INTERNAL_ANON_SUBTREE,
    KEY_BACKLINKS,
    KEY_TRANSPARENT_BACKLINKS,
    KEY_REFERENCES,
    KEY_COLLECT,
    KEY_ASREF,
    KEY_ASBACK,
    KEY_FOOTER_MODE,
    KEY_FOOTER_SORT_BY,
];

pub fn is_plain_metadata(s: &str) -> bool {
    PLAIN_METADATA.contains(&s)
}

fn parse_bool_value(key: &str, value: &str, slug: Slug) -> eyre::Result<bool> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(eyre!(
            "invalid bool metadata in `{}`: `{}` = `{}` (expected `true` or `false`)",
            slug,
            key,
            value
        )),
    }
}

fn parse_footer_mode_value(key: &str, value: &str, slug: Slug) -> eyre::Result<FooterMode> {
    value.parse().map_err(|_| {
        eyre!(
            "invalid metadata in `{}`: `{}` = `{}` (expected `embed` or `link`)",
            slug,
            key,
            value
        )
    })
}

fn parse_footer_sort_by(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

/// The built-in (plain) metadata fields, parsed and validated once at
/// construction time instead of on every read.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BuiltinMeta {
    pub slug: Option<Slug>,
    pub ext: Option<String>,
    pub parent: Option<Slug>,
    pub page_title: Option<String>,
    pub data_taxon: Option<String>,
    pub source_slug: Option<String>,
    pub source_pos: Option<String>,
    /// Default: `true`
    pub backlinks: bool,
    /// Default: `false`
    pub transparent_backlinks: bool,
    /// Default: `true`
    pub references: bool,
    /// Default: `false`
    pub collect: bool,
    /// Default: `false`
    pub internal_anon_subtree: bool,
    /// No baked default: falls back to `environment::asref()` at the call site.
    pub asref: Option<bool>,
    /// No baked default: falls back to `true` at the call site.
    pub asback: Option<bool>,
    pub footer_mode: Option<FooterMode>,
    pub footer_sort_by: Option<String>,
}

impl Default for BuiltinMeta {
    fn default() -> Self {
        BuiltinMeta {
            slug: None,
            ext: None,
            parent: None,
            page_title: None,
            data_taxon: None,
            source_slug: None,
            source_pos: None,
            backlinks: true,
            transparent_backlinks: false,
            references: true,
            collect: false,
            internal_anon_subtree: false,
            asref: None,
            asback: None,
            footer_mode: None,
            footer_sort_by: None,
        }
    }
}

impl BuiltinMeta {
    /// Assign a plain built-in metadata key, validating its value.
    ///
    /// Returns `Ok(true)` when `key` is a built-in key (and thus consumed),
    /// `Ok(false)` when it is not (the caller should treat it as fancy/custom).
    pub fn assign(&mut self, key: &str, value: &str, slug: Slug) -> eyre::Result<bool> {
        match key {
            KEY_SLUG => self.slug = Some(Slug::new(value)),
            KEY_EXT => self.ext = Some(value.to_string()),
            KEY_PARENT => self.parent = Some(Slug::new(value)),
            KEY_PAGE_TITLE => self.page_title = Some(value.to_string()),
            KEY_DATA_TAXON => self.data_taxon = Some(value.to_string()),
            KEY_SOURCE_SLUG => self.source_slug = Some(value.to_string()),
            KEY_SOURCE_POS => self.source_pos = Some(value.to_string()),
            KEY_BACKLINKS => self.backlinks = parse_bool_value(key, value, slug)?,
            KEY_TRANSPARENT_BACKLINKS => {
                self.transparent_backlinks = parse_bool_value(key, value, slug)?
            }
            KEY_REFERENCES => self.references = parse_bool_value(key, value, slug)?,
            KEY_COLLECT => self.collect = parse_bool_value(key, value, slug)?,
            KEY_INTERNAL_ANON_SUBTREE => {
                self.internal_anon_subtree = parse_bool_value(key, value, slug)?
            }
            KEY_ASREF => self.asref = Some(parse_bool_value(key, value, slug)?),
            KEY_ASBACK => self.asback = Some(parse_bool_value(key, value, slug)?),
            KEY_FOOTER_MODE => self.footer_mode = Some(parse_footer_mode_value(key, value, slug)?),
            KEY_FOOTER_SORT_BY => self.footer_sort_by = parse_footer_sort_by(value),
            _ => return Ok(false),
        }
        Ok(true)
    }
}

/// Section metadata. The built-in fields live in [`BuiltinMeta`], while the two
/// rich fields (`title`, `taxon`) and all custom keys are generic over the value
/// type: [`HTMLContent`] before resolution, `String` after.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MetaData<V: Clone> {
    pub builtin: BuiltinMeta,
    pub title: Option<V>,
    pub taxon: Option<V>,
    pub custom: OrderedMap<String, V>,
}

/// Unresolved (parse-stage) metadata. `title`/`taxon` and custom values may
/// still contain unresolved embeds/local links.
pub type HTMLMetaData = MetaData<HTMLContent>;

/// Compiled metadata. Every value has been resolved to a plain string.
pub type EntryMetaData = MetaData<String>;

impl<V: Clone> Default for MetaData<V> {
    fn default() -> Self {
        MetaData {
            builtin: BuiltinMeta::default(),
            title: None,
            taxon: None,
            custom: OrderedMap::new(),
        }
    }
}

impl<V: Clone> MetaData<V> {
    pub fn title(&self) -> Option<&V> {
        self.title.as_ref()
    }

    pub fn taxon(&self) -> Option<&V> {
        self.taxon.as_ref()
    }

    pub fn data_taxon(&self) -> Option<&str> {
        self.builtin.data_taxon.as_deref()
    }

    pub fn page_title(&self) -> Option<&str> {
        self.builtin.page_title.as_deref()
    }

    pub fn parent(&self) -> Option<Slug> {
        self.builtin.parent
    }

    pub fn slug(&self) -> Option<Slug> {
        self.builtin.slug
    }

    pub fn ext(&self) -> Option<&str> {
        self.builtin.ext.as_deref()
    }

    pub fn source_slug(&self) -> Option<&str> {
        self.builtin.source_slug.as_deref()
    }

    pub fn source_pos(&self) -> Option<&str> {
        self.builtin.source_pos.as_deref()
    }

    pub fn backlinks_enabled(&self) -> bool {
        self.builtin.backlinks
    }

    pub fn is_backlinks_transparent(&self) -> bool {
        self.builtin.transparent_backlinks
    }

    pub fn references_enabled(&self) -> bool {
        self.builtin.references
    }

    pub fn is_collect(&self) -> bool {
        self.builtin.collect
    }

    pub fn internal_anon_subtree(&self) -> bool {
        self.builtin.internal_anon_subtree
    }

    pub fn is_asref(&self) -> Option<bool> {
        self.builtin.asref
    }

    pub fn is_asback(&self) -> Option<bool> {
        self.builtin.asback
    }

    pub fn footer_mode(&self) -> Option<FooterMode> {
        self.builtin.footer_mode
    }

    pub fn footer_sort_by(&self) -> Option<String> {
        self.builtin.footer_sort_by.clone()
    }

    /// Return all custom metadata values.
    pub fn etc(&self) -> Vec<V> {
        self.custom.values().cloned().collect()
    }

    pub fn id(&self) -> eyre::Result<String> {
        let slug = self
            .builtin
            .slug
            .ok_or_else(|| eyre!("missing required metadata `slug` while rendering section id"))?;
        Ok(crate::slug::to_hash_id(slug.as_str()))
    }
}

impl MetaData<String> {
    /// Read a custom metadata key by name.
    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.custom.get(key).map(String::as_str)
    }
}

impl HTMLMetaData {
    pub fn with_slug_ext(slug: Slug, ext: impl Into<String>) -> HTMLMetaData {
        HTMLMetaData {
            builtin: BuiltinMeta {
                slug: Some(slug),
                ext: Some(ext.into()),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn compute_textual_attrs(&mut self) {
        if self.builtin.page_title.is_none() {
            if let Some(title) = &self.title {
                self.builtin.page_title = Some(title.remove_all_tags());
            }
        }

        if self.builtin.data_taxon.is_none() {
            if let Some(taxon) = &self.taxon {
                self.builtin.data_taxon =
                    Some(Taxon::to_data_taxon(&taxon.remove_all_tags()).to_string());
            }
        }
    }
}

impl EntryMetaData {
    pub fn to_header(
        &self,
        adhoc_title: Option<&str>,
        adhoc_taxon: Option<&str>,
    ) -> eyre::Result<String> {
        let entry_taxon = self.taxon().map_or("", |s| s.as_str());
        let taxon = adhoc_taxon.unwrap_or(entry_taxon);
        let entry_title = self.title().map_or("", |s| s.as_str());
        let title = adhoc_title.unwrap_or(entry_title);
        let slug = self
            .slug()
            .ok_or_else(|| eyre!("missing required metadata `slug` while rendering header"))?;
        let ext = self.ext().ok_or_else(|| {
            eyre!(
                "missing required metadata `ext` while rendering header for `{}`",
                slug
            )
        })?;
        let show_slug = !self.builtin.internal_anon_subtree;
        let etc = self.etc();

        Ok(html_flake::html_header(html_flake::HtmlHeaderArgs {
            title,
            taxon,
            slug: &slug,
            ext,
            show_slug,
            source_slug: self.source_slug(),
            source_pos: self.source_pos(),
            etc: &etc,
        }))
    }

    /// hidden suffix `/index` in slug text.
    pub fn to_slug_text(slug: &str) -> String {
        let mut slug_text = slug.strip_suffix("/index").unwrap_or(slug);
        if environment::is_short_slug() {
            slug_text = slug_text
                .rsplit_once('/')
                .map_or(slug_text, |(_, rest)| rest);
        }
        slug_text.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assign_rejects_invalid_bool() {
        let mut builtin = BuiltinMeta::default();
        let err = builtin
            .assign(KEY_REFERENCES, "maybe", Slug::new("a"))
            .unwrap_err();
        assert!(err.to_string().contains("invalid bool metadata"));
    }
}
