// Copyright (c) 2025 Kodama Project. All rights reserved.
// Released under the GPL-3.0 license as described in the file LICENSE.
// Authors: Kokic (@kokic), Alias Qli (@AliasQli), Spore (@s-cerevisiae)

use std::collections::{HashSet, VecDeque};

use camino::Utf8PathBuf;

use crate::{
    environment,
    slug::{SectionKind, Slug, SourceRole},
};

use super::{state, DirtySet, Workspace};

pub(super) fn source_relative_path(slug: Slug, kind: SectionKind) -> Utf8PathBuf {
    Utf8PathBuf::from(format!("{}.{}", slug, environment::suffix_for_kind(kind)))
}

pub fn expand_dirty_paths(workspace: &Workspace, dirty_paths: &DirtySet) -> DirtySet {
    let mut expanded = dirty_paths.clone();
    let source_paths: HashSet<Utf8PathBuf> = workspace
        .slug_exts
        .iter()
        .map(|(&slug, &kind)| source_relative_path(slug, kind))
        .collect();

    let mut dirty_all_sources = false;
    let mut dirty_all_typst_sources = false;
    for path in dirty_paths {
        let is_known_source = source_paths.contains(path);
        let role = path
            .extension()
            .map(environment::classify_source)
            .unwrap_or(SourceRole::Unknown);
        match role {
            SourceRole::Markdown | SourceRole::Typst if is_known_source => {}
            SourceRole::TypstLib => dirty_all_typst_sources = true,
            _ => {
                // Unknown tree-side dependency (e.g. include file): conservatively reparse all.
                dirty_all_sources = true;
            }
        }
    }

    if dirty_all_sources {
        workspace.slug_exts.iter().for_each(|(&slug, &kind)| {
            expanded.insert(source_relative_path(slug, kind));
        });
        return expanded;
    }

    if dirty_all_typst_sources {
        workspace
            .slug_exts
            .iter()
            .filter(|(_, &kind)| matches!(kind, SectionKind::Typst))
            .for_each(|(&slug, &kind)| {
                expanded.insert(source_relative_path(slug, kind));
            });
    }

    expanded
}

pub(super) fn dirty_source_slugs(workspace: &Workspace, dirty_paths: &DirtySet) -> HashSet<Slug> {
    workspace
        .slug_exts
        .iter()
        .filter_map(|(&slug, &kind)| {
            let relative = source_relative_path(slug, kind);
            dirty_paths.contains(relative.as_path()).then_some(slug)
        })
        .collect()
}

pub(super) fn affected_slugs_from_dirty(
    state: &state::CompileState,
    dirty_source_slugs: &HashSet<Slug>,
) -> HashSet<Slug> {
    let mut affected = dirty_source_slugs.clone();

    // Descendant sections (embedded children) share source ownership with their parent
    // in subtree mode, so source-dirty must include the whole descendant chain.
    let mut changed = true;
    while changed {
        changed = false;
        for (&slug, callback) in &state.callback().0 {
            if callback.parent != slug
                && affected.contains(&callback.parent)
                && affected.insert(slug)
            {
                changed = true;
            }
        }
    }

    let mut queue: VecDeque<Slug> = affected.iter().copied().collect();

    // If a linker page changes, the target's backlink list changes too.
    for (&target_slug, callback) in &state.callback().0 {
        if callback
            .backlinks
            .iter()
            .any(|backlink_slug| dirty_source_slugs.contains(backlink_slug))
            && affected.insert(target_slug)
        {
            queue.push_back(target_slug);
        }
    }

    while let Some(slug) = queue.pop_front() {
        let Some(callback) = state.callback().0.get(&slug) else {
            continue;
        };

        if callback.parent != slug && affected.insert(callback.parent) {
            queue.push_back(callback.parent);
        }

        for &backlink_slug in &callback.backlinks {
            if affected.insert(backlink_slug) {
                queue.push_back(backlink_slug);
            }
        }
    }

    affected
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use super::*;
    use crate::{
        compiler::section::{
            EmbedContent, HTMLContent, LazyContent, LocalLink, SectionOption, UnresolvedSection,
        },
        entry::{HTMLMetaData, KEY_EXT, KEY_PAGE_TITLE, KEY_SLUG, KEY_TITLE},
        ordered_map::OrderedMap,
    };

    fn shallow(slug: &str, content: HTMLContent) -> UnresolvedSection {
        let mut metadata = OrderedMap::new();
        metadata.insert(KEY_SLUG.to_string(), HTMLContent::Plain(slug.to_string()));
        metadata.insert(KEY_EXT.to_string(), HTMLContent::Plain("md".to_string()));
        metadata.insert(KEY_TITLE.to_string(), HTMLContent::Plain(slug.to_string()));
        metadata.insert(
            KEY_PAGE_TITLE.to_string(),
            HTMLContent::Plain(slug.to_string()),
        );
        UnresolvedSection {
            metadata: HTMLMetaData(metadata),
            content,
        }
    }

    fn with_test_env(f: impl FnOnce()) {
        let root = crate::test_io::case_dir("incremental");
        crate::environment::with_test_environment(
            root.clone(),
            crate::environment::BuildMode::Publish,
            f,
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn test_expand_dirty_paths_typst_dependency_marks_all_typst_sources() {
        with_test_env(|| {
            let mut slug_exts = HashMap::new();
            slug_exts.insert(Slug::new("a"), SectionKind::Markdown);
            slug_exts.insert(Slug::new("b"), SectionKind::Typst);
            slug_exts.insert(Slug::new("c"), SectionKind::Typst);
            let workspace = Workspace { slug_exts };

            let mut dirty = DirtySet::new();
            dirty.insert(Utf8PathBuf::from("shared.typ"));

            let expanded = expand_dirty_paths(&workspace, &dirty);
            assert!(expanded.contains(&Utf8PathBuf::from("shared.typ")));
            assert!(expanded.contains(&Utf8PathBuf::from("b.typst")));
            assert!(expanded.contains(&Utf8PathBuf::from("c.typst")));
            assert!(!expanded.contains(&Utf8PathBuf::from("a.md")));
        });
    }

    #[test]
    fn test_expand_dirty_paths_typst_source_change_keeps_scope_local() {
        with_test_env(|| {
            let mut slug_exts = HashMap::new();
            slug_exts.insert(Slug::new("a"), SectionKind::Markdown);
            slug_exts.insert(Slug::new("b"), SectionKind::Typst);
            slug_exts.insert(Slug::new("c"), SectionKind::Typst);
            let workspace = Workspace { slug_exts };

            let mut dirty = DirtySet::new();
            dirty.insert(Utf8PathBuf::from("b.typst"));

            let expanded = expand_dirty_paths(&workspace, &dirty);
            assert!(expanded.contains(&Utf8PathBuf::from("b.typst")));
            assert!(!expanded.contains(&Utf8PathBuf::from("c.typst")));
            assert!(!expanded.contains(&Utf8PathBuf::from("a.md")));
        });
    }

    #[test]
    fn test_expand_dirty_paths_unknown_markdown_dependency_marks_all_sources() {
        with_test_env(|| {
            let mut slug_exts = HashMap::new();
            slug_exts.insert(Slug::new("a"), SectionKind::Markdown);
            slug_exts.insert(Slug::new("b"), SectionKind::Typst);
            let workspace = Workspace { slug_exts };

            let mut dirty = DirtySet::new();
            dirty.insert(Utf8PathBuf::from("_includes/shared.md"));

            let expanded = expand_dirty_paths(&workspace, &dirty);
            assert!(expanded.contains(&Utf8PathBuf::from("a.md")));
            assert!(expanded.contains(&Utf8PathBuf::from("b.typst")));
        });
    }

    #[test]
    fn test_expand_dirty_paths_unknown_tree_file_marks_all_sources() {
        with_test_env(|| {
            let mut slug_exts = HashMap::new();
            slug_exts.insert(Slug::new("a"), SectionKind::Markdown);
            slug_exts.insert(Slug::new("b"), SectionKind::Typst);
            let workspace = Workspace { slug_exts };

            let mut dirty = DirtySet::new();
            dirty.insert(Utf8PathBuf::from("includes/snippet.txt"));

            let expanded = expand_dirty_paths(&workspace, &dirty);
            assert!(expanded.contains(&Utf8PathBuf::from("a.md")));
            assert!(expanded.contains(&Utf8PathBuf::from("b.typst")));
        });
    }

    #[test]
    fn test_dirty_source_slugs_maps_relative_paths_to_slug_ids() {
        with_test_env(|| {
            let mut slug_exts = HashMap::new();
            slug_exts.insert(Slug::new("a"), SectionKind::Markdown);
            slug_exts.insert(Slug::new("b"), SectionKind::Typst);
            let workspace = Workspace { slug_exts };

            let mut dirty = DirtySet::new();
            dirty.insert(Utf8PathBuf::from("a.md"));
            dirty.insert(Utf8PathBuf::from("unknown.txt"));

            let dirty_slugs = dirty_source_slugs(&workspace, &dirty);
            assert!(dirty_slugs.contains(&Slug::new("a")));
            assert!(!dirty_slugs.contains(&Slug::new("b")));
        });
    }

    #[test]
    fn test_affected_slugs_include_link_targets_when_linker_changes() {
        with_test_env(|| {
            let mut shallows = HashMap::new();
            shallows.insert(
                Slug::new("a"),
                shallow(
                    "a",
                    HTMLContent::Lazy(vec![LazyContent::Local(LocalLink {
                        url: "/b.md".to_string(),
                        text: None,
                    })]),
                ),
            );
            shallows.insert(
                Slug::new("b"),
                shallow("b", HTMLContent::Plain("<p>b</p>".to_string())),
            );

            let state = state::compile_all(&shallows).unwrap();
            let dirty_slugs = HashSet::from([Slug::new("a")]);
            let affected = affected_slugs_from_dirty(&state, &dirty_slugs);

            assert!(affected.contains(&Slug::new("a")));
            assert!(affected.contains(&Slug::new("b")));
        });
    }

    #[test]
    fn test_affected_slugs_include_embedded_descendants_when_parent_source_changes() {
        with_test_env(|| {
            let mut shallows = HashMap::new();
            shallows.insert(
                Slug::new("root"),
                shallow(
                    "root",
                    HTMLContent::Lazy(vec![LazyContent::Embed(EmbedContent {
                        url: "/child.md".to_string(),
                        title: None,
                        option: SectionOption::default(),
                    })]),
                ),
            );
            shallows.insert(
                Slug::new("child"),
                shallow(
                    "child",
                    HTMLContent::Lazy(vec![LazyContent::Embed(EmbedContent {
                        url: "/leaf.md".to_string(),
                        title: None,
                        option: SectionOption::default(),
                    })]),
                ),
            );
            shallows.insert(
                Slug::new("leaf"),
                shallow("leaf", HTMLContent::Plain("<p>leaf</p>".to_string())),
            );

            let state = state::compile_all(&shallows).unwrap();
            let dirty_slugs = HashSet::from([Slug::new("root")]);
            let affected = affected_slugs_from_dirty(&state, &dirty_slugs);

            assert!(affected.contains(&Slug::new("root")));
            assert!(affected.contains(&Slug::new("child")));
            assert!(affected.contains(&Slug::new("leaf")));
        });
    }
}
