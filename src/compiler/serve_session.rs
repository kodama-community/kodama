// Copyright (c) 2025 Kodama Project. All rights reserved.
// Released under the GPL-3.0 license as described in the file LICENSE.
// Authors: Kokic (@kokic), Alias Qli (@AliasQli), Spore (@s-cerevisiae)

use std::collections::{HashMap, HashSet};

use eyre::{eyre, WrapErr};

use crate::{
    environment,
    slug::{SectionKind, Slug},
    typst_cache,
};

use super::section::UnresolvedSection;
use super::{
    incremental::{dirty_source_slugs, source_relative_path},
    inline_typst, parse_source_sections,
    stale::cleanup_stale_slug_artifacts,
    state::CompileCache,
    write_compiled_state, write_entry_cache, CompileOutputs, DirtySet, ParsedSections, SourceCache,
    SourceDependencies, Workspace,
};

#[derive(Default)]
pub struct ServeCompileSession {
    initialized: bool,
    shallows: HashMap<Slug, UnresolvedSection>,
    source_sections: HashMap<Slug, Vec<Slug>>,
    source_dependencies: SourceDependencies,
    compiled: Option<CompileCache>,
}

impl ServeCompileSession {
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    pub fn compile_full(
        &mut self,
        workspace: Workspace,
        outputs: CompileOutputs,
    ) -> eyre::Result<()> {
        self.initialized = false;
        self.compiled = None;
        cleanup_stale_slug_artifacts(&workspace)
            .wrap_err("failed to clean stale slug artifacts")?;
        let (shallows, source_sections, dependencies) =
            super::collect_shallows_with_sources(&workspace, None)?;
        self.shallows = shallows;
        self.source_sections = source_sections;
        self.source_dependencies = dependencies;
        self.compile_sections(&HashSet::new(), outputs, true)
    }

    pub fn compile_incremental(
        &mut self,
        workspace: Workspace,
        dirty_paths: &DirtySet,
        outputs: CompileOutputs,
    ) -> eyre::Result<()> {
        if !self.initialized {
            return self.compile_full(workspace, outputs);
        }
        self.initialized = false;
        let mut changed_sections = cleanup_stale_slug_artifacts(&workspace)
            .wrap_err("failed to clean stale slug artifacts")?;

        let removed_sources: Vec<Slug> = self
            .source_sections
            .keys()
            .copied()
            .filter(|source_slug| !workspace.slug_exts.contains_key(source_slug))
            .collect();
        for source_slug in removed_sources {
            self.source_dependencies.remove(&source_slug);
            if let Some(slugs) = self.source_sections.remove(&source_slug) {
                for slug in slugs {
                    self.shallows.remove(&slug);
                    changed_sections.insert(slug);
                }
            }
        }

        let mut parse_targets = dirty_source_slugs(&workspace, dirty_paths);

        for (&slug, &kind) in &workspace.slug_exts {
            let dependencies_current = match self.source_dependencies.get(&slug) {
                Some(dependencies) => dependencies.is_current()?,
                None => false,
            };
            if self.needs_refresh(slug, kind) || !dependencies_current {
                parse_targets.insert(slug);
            }
        }

        inline_typst::begin_inline_batch();
        let mut parsed: Vec<(Slug, ParsedSections, SourceCache)> = Vec::new();
        for slug in parse_targets {
            let Some(&kind) = workspace.slug_exts.get(&slug) else {
                continue;
            };
            let relative_path = source_relative_path(slug, kind);
            let entry_path = environment::entry_file_path(relative_path.as_path());
            let (sections, dependencies) =
                typst_cache::capture_dependencies(|| parse_source_sections(slug, kind))?;
            let source_meta = environment::relative_source_meta(&relative_path);
            parsed.push((
                slug,
                sections,
                SourceCache {
                    entry_path,
                    source_meta,
                    typst_dependencies: dependencies,
                    needs_write: true,
                },
            ));
        }

        // Compile all freshly parsed inline formulas in one batch, then resolve
        // their placeholders and write the entry caches.
        let batch = inline_typst::compile_inline_batch()?;
        for (source_slug, mut sections, mut cache) in parsed {
            for (_, section) in &mut sections {
                inline_typst::resolve_inline_typst_section(section, &batch.contents);
            }
            if let Some(dependencies) = batch.dependencies_for(source_slug) {
                cache.typst_dependencies.merge(dependencies);
            }
            write_entry_cache(
                cache.entry_path.as_path(),
                &sections,
                cache.source_meta,
                &cache.typst_dependencies,
            )?;
            self.source_dependencies
                .insert(source_slug, cache.typst_dependencies);

            if let Some(previous_slugs) = self.source_sections.remove(&source_slug) {
                for previous_slug in previous_slugs {
                    self.shallows.remove(&previous_slug);
                    changed_sections.insert(previous_slug);
                }
            }

            let produced_slugs: Vec<Slug> = sections
                .iter()
                .map(|(section_slug, _)| *section_slug)
                .collect();
            for (section_slug, section) in sections {
                if let Some(owner) = self.slug_owner(section_slug) {
                    if owner != source_slug {
                        return Err(eyre!(
                            "section slug collision: `{}` is generated by both `{}` and `{}`",
                            section_slug,
                            owner,
                            source_slug
                        ));
                    }
                }
                self.shallows.insert(section_slug, section);
                changed_sections.insert(section_slug);
            }
            self.source_sections.insert(source_slug, produced_slugs);
        }
        self.compile_sections(&changed_sections, outputs, false)
    }

    pub fn rewrite_all_from_memory(&mut self, outputs: CompileOutputs) -> eyre::Result<()> {
        if !self.initialized {
            let workspace = super::all_trees_source(&environment::trees_dir())?;
            return self.compile_full(workspace, outputs);
        }
        for dependencies in self.source_dependencies.values() {
            if !dependencies.is_current()? {
                let workspace = super::all_trees_source(&environment::trees_dir())?;
                return self.compile_full(workspace, outputs);
            }
        }
        // Global/template/theme changes can require re-emitting all pages,
        // but they don't require reparsing source trees.
        self.initialized = false;
        self.compile_sections(&HashSet::new(), outputs, true)
    }

    fn compile_sections(
        &mut self,
        changed: &HashSet<Slug>,
        outputs: CompileOutputs,
        write_all: bool,
    ) -> eyre::Result<()> {
        let (state, affected) = self
            .compiled
            .take()
            .unwrap_or_default()
            .compile(&self.shallows, changed)?;
        let mut slugs: Vec<Slug> = if write_all {
            state.compiled().keys().copied().collect()
        } else {
            affected.into_iter().collect()
        };
        slugs.sort();
        write_compiled_state(&self.shallows, &state, slugs, outputs)?;
        self.compiled = Some(state.into_cache());
        self.initialized = true;
        Ok(())
    }

    fn needs_refresh(&self, slug: Slug, kind: SectionKind) -> bool {
        if !self.source_sections.contains_key(&slug) {
            return true;
        }
        let Some(shallow) = self.shallows.get(&slug) else {
            return true;
        };
        let expected_suffix = environment::suffix_for_kind(kind);
        shallow
            .ext()
            .map(|value| value != expected_suffix.as_str())
            .unwrap_or(true)
    }

    fn slug_owner(&self, slug: Slug) -> Option<Slug> {
        self.source_sections
            .iter()
            .find_map(|(source_slug, slugs)| slugs.contains(&slug).then_some(*source_slug))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use camino::{Utf8Path, Utf8PathBuf};

    use crate::{
        compiler::{
            artifacts::graph_snapshot,
            section::{HTMLContent, UnresolvedSection},
            state,
            writer::Writer,
        },
        entry::HTMLMetaData,
    };

    use super::*;

    fn shallow(slug: &str, ext: &str) -> UnresolvedSection {
        UnresolvedSection {
            metadata: HTMLMetaData::with_slug_ext(Slug::new(slug), ext),
            content: HTMLContent::Plain(String::new()),
        }
    }

    fn with_site(f: impl FnOnce(&Utf8Path)) {
        let root = crate::test_io::case_dir("incremental-session");
        fs::create_dir_all(root.join("trees")).unwrap();
        let mut config = crate::config::Config::default();
        config.serve.edit = None;
        environment::with_test_environment_config(
            root.clone(),
            environment::BuildMode::Serve,
            config,
            || f(&root),
        );
        let _ = fs::remove_dir_all(root);
    }

    fn write_source(root: &Utf8Path, relative: &str, content: &str) {
        fs::write(root.join("trees").join(relative), content).unwrap();
    }

    fn build(session: &mut ServeCompileSession, dirty: Option<&str>) -> eyre::Result<()> {
        typst_cache::begin_build();
        let workspace = super::super::all_trees_source(&environment::trees_dir())?;
        match dirty {
            Some(path) => session.compile_incremental(
                workspace,
                &DirtySet::from([Utf8PathBuf::from(path)]),
                CompileOutputs::default(),
            ),
            None => session.compile_full(workspace, CompileOutputs::default()),
        }
    }

    fn assert_outputs_match_full(session: &ServeCompileSession) {
        let full = state::compile_all(&session.shallows).unwrap();
        for (&slug, section) in full.compiled() {
            let expected = Writer::html_doc(section, &full).unwrap().0;
            let path = environment::output_path(format!("{slug}.html"));
            assert_eq!(fs::read_to_string(path).unwrap(), expected, "page {slug}");
        }
        let graph: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(environment::graph_path(&environment::output_dir())).unwrap(),
        )
        .unwrap();
        assert_eq!(graph, serde_json::to_value(graph_snapshot(&full)).unwrap());
    }

    #[test]
    fn incremental_outputs_match_full_after_link_changes_and_deletions() {
        with_site(|root| {
            write_source(
                root,
                "index.md",
                "---\ntitle: Root\n---\n\n[](/branch.md#:embed)\n",
            );
            write_source(root, "branch.md", "# Branch\n\n[Target](/target.md)\n");
            write_source(root, "target.md", "# Target\n\nTarget content\n");
            write_source(root, "other.md", "# Other\n\nUnrelated content\n");
            let mut session = ServeCompileSession::default();
            build(&mut session, None).unwrap();
            let other = environment::output_path("other.html");
            let old_time =
                std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_600_000_000);
            fs::OpenOptions::new()
                .write(true)
                .open(&other)
                .unwrap()
                .set_modified(old_time)
                .unwrap();
            let other_modified = fs::metadata(&other).unwrap().modified().unwrap();

            write_source(root, "branch.md", "# Renamed Branch\n\nNo link\n");
            build(&mut session, Some("branch.md")).unwrap();
            assert_outputs_match_full(&session);
            assert_eq!(
                fs::metadata(&other).unwrap().modified().unwrap(),
                other_modified
            );

            fs::remove_file(root.join("trees/target.md")).unwrap();
            build(&mut session, Some("target.md")).unwrap();
            assert_outputs_match_full(&session);

            write_source(
                root,
                "index.md",
                "---\ntitle: Renamed Root\n---\n\n[](/branch.md#:embed)\n",
            );
            build(&mut session, Some("index.md")).unwrap();
            assert_outputs_match_full(&session);
            assert_ne!(
                fs::metadata(other).unwrap().modified().unwrap(),
                other_modified
            );
        });
    }

    #[test]
    fn generated_subsections_and_global_templates_match_full_outputs() {
        with_site(|root| {
            write_source(
                root,
                "index.md",
                "# Root\n\n<block slug=\"child\" title=\"Child\">\nBefore\n</block>\n",
            );
            write_source(root, "other.md", "# Other\n\nUnrelated\n");
            let mut session = ServeCompileSession::default();
            build(&mut session, None).unwrap();
            write_source(root, "index.md", "# Root\n\n<block slug=\"child\" title=\"Child\">\nAfter\n</block>\n\n<remark title=\"Anonymous\">\nNested\n</remark>\n");
            build(&mut session, Some("index.md")).unwrap();
            assert_outputs_match_full(&session);

            write_source(root, "index.md", "# Root\n\nSubsections removed\n");
            build(&mut session, Some("index.md")).unwrap();
            assert!(!session.shallows.contains_key(&Slug::new("child")));
            assert_outputs_match_full(&session);

            fs::write(
                root.join("import-style.html"),
                "<style>body { color: red; }</style>",
            )
            .unwrap();
            session
                .rewrite_all_from_memory(CompileOutputs::default())
                .unwrap();
            assert_outputs_match_full(&session);
            assert!(fs::read_to_string(environment::output_path("other.html"))
                .unwrap()
                .contains("color: red"));
        });
    }

    #[test]
    fn failed_incremental_build_recovers_without_reusing_partial_state() {
        with_site(|root| {
            write_source(root, "index.md", "# Root\n\n[](/child.md#:embed)\n");
            write_source(root, "child.md", "# Child\n\nBefore\n");
            let mut session = ServeCompileSession::default();
            build(&mut session, None).unwrap();

            write_source(root, "child.md", "# Child\n\n[](/index.md#:embed)\n");
            assert!(build(&mut session, Some("child.md")).is_err());
            assert!(!session.is_initialized());

            write_source(root, "child.md", "# Child\n\nRecovered\n");
            build(&mut session, Some("child.md")).unwrap();
            assert!(session.is_initialized());
            assert_outputs_match_full(&session);
        });
    }

    #[test]
    fn test_needs_refresh_when_shallow_missing() {
        let session = ServeCompileSession::default();
        assert!(session.needs_refresh(Slug::new("a"), SectionKind::Markdown));
    }

    #[test]
    fn test_needs_refresh_when_extension_matches() {
        let mut session = ServeCompileSession::default();
        session
            .source_sections
            .insert(Slug::new("a"), vec![Slug::new("a")]);
        session.shallows.insert(Slug::new("a"), shallow("a", "md"));
        assert!(!session.needs_refresh(Slug::new("a"), SectionKind::Markdown));
    }

    #[test]
    fn test_needs_refresh_when_extension_differs() {
        let mut session = ServeCompileSession::default();
        session
            .source_sections
            .insert(Slug::new("a"), vec![Slug::new("a")]);
        session.shallows.insert(Slug::new("a"), shallow("a", "md"));
        assert!(session.needs_refresh(Slug::new("a"), SectionKind::Typst));
    }
}
