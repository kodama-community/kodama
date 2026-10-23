// Copyright (c) 2026 Kodama Project. All rights reserved.
// Released under the GPL-3.0 license as described in the file LICENSE.

use super::*;

#[derive(Debug)]
pub(super) enum CompileEvent {
    Embed(Slug),
    Callback(Callback),
}

#[derive(Debug, Default)]
pub(super) struct SectionRelations {
    pub dependencies: HashSet<Slug>,
    // Replay in compilation order so implicit parent selection stays unchanged.
    pub events: Vec<CompileEvent>,
}

#[derive(Debug, Default)]
pub(in crate::compiler) struct CompileCache {
    pub(super) sections: HashMap<Slug, Section>,
    pub(super) relations: HashMap<Slug, SectionRelations>,
    callback: Callback,
}

impl CompileCache {
    pub(in crate::compiler) fn compile(
        mut self,
        shallows: &UnresolvedSections,
        changed: &HashSet<Slug>,
    ) -> eyre::Result<(CompileState, HashSet<Slug>)> {
        let previous_callback = std::mem::take(&mut self.callback);
        let previous_sections = self.invalidate(changed);
        let state = compile_with_cache(shallows, true, Some(self))?;
        let affected = affected_pages(&state, changed, &previous_sections, &previous_callback);
        Ok((state, affected))
    }

    fn invalidate(&mut self, changed: &HashSet<Slug>) -> HashMap<Slug, Section> {
        if changed.is_empty() {
            return HashMap::new();
        }
        let mut dependents: HashMap<Slug, Vec<Slug>> = HashMap::new();
        for (&owner, relations) in &self.relations {
            for &dependency in &relations.dependencies {
                dependents.entry(dependency).or_default().push(owner);
            }
        }

        let mut invalidated = changed.clone();
        let mut pending: Vec<Slug> = changed.iter().copied().collect();
        while let Some(slug) = pending.pop() {
            if let Some(users) = dependents.get(&slug) {
                for &user in users {
                    if invalidated.insert(user) {
                        pending.push(user);
                    }
                }
            }
        }

        let mut previous = HashMap::new();
        for slug in invalidated {
            if let Some(section) = self.sections.remove(&slug) {
                previous.insert(slug, section);
            }
            self.relations.remove(&slug);
        }
        previous
    }
}

impl CompileState {
    pub(super) fn rebuild_callback(&mut self) {
        fn replay(
            slug: Slug,
            relations: &HashMap<Slug, SectionRelations>,
            callback: &mut Callback,
            visited: &mut HashSet<Slug>,
        ) {
            let Some(record) = relations.get(&slug) else {
                return;
            };
            if record.events.is_empty() || !visited.insert(slug) {
                return;
            }
            for event in &record.events {
                match event {
                    CompileEvent::Embed(child) => replay(*child, relations, callback, visited),
                    CompileEvent::Callback(value) => callback.merge(value.clone()),
                }
            }
        }

        let mut roots: Vec<Slug> = self
            .relations
            .iter()
            .filter_map(|(&slug, record)| (!record.events.is_empty()).then_some(slug))
            .collect();
        roots.sort();
        let mut visited = HashSet::new();
        replay(
            Slug::new("index"),
            &self.relations,
            &mut self.callback,
            &mut visited,
        );
        for slug in roots {
            replay(slug, &self.relations, &mut self.callback, &mut visited);
        }
    }

    pub(in crate::compiler) fn into_cache(mut self) -> CompileCache {
        self.compiled.extend(self.hidden_sections);
        for (slug, references) in self.hidden_references {
            if let Some(section) = self.compiled.get_mut(&slug) {
                section.references.extend(references);
            }
        }
        CompileCache {
            sections: self.compiled,
            relations: self.relations,
            callback: self.callback,
        }
    }
}

fn affected_pages(
    state: &CompileState,
    changed: &HashSet<Slug>,
    previous_sections: &HashMap<Slug, Section>,
    previous_callback: &Callback,
) -> HashSet<Slug> {
    let mut affected = state.recompiled.clone();
    let changed_sections: HashSet<Slug> = changed.union(&state.recompiled).copied().collect();
    let changed_headers: HashSet<Slug> = changed_sections
        .iter()
        .copied()
        .filter(|slug| {
            fn header(section: &Section) -> (Option<&str>, Option<&str>) {
                (
                    section.metadata.title().map(String::as_str),
                    section.metadata.page_title(),
                )
            }
            previous_sections.get(slug).map(header) != state.compiled.get(slug).map(header)
        })
        .collect();

    for &slug in previous_callback.0.keys().chain(state.callback.0.keys()) {
        if previous_callback.0.get(&slug) != state.callback.0.get(&slug) {
            affected.insert(slug);
        }
    }

    for (&slug, section) in &state.compiled {
        let callback = state.callback.0.get(&slug);
        let parent = callback.map_or(Slug::new("index"), |value| value.parent);
        if changed_headers.contains(&parent)
            || section
                .references
                .iter()
                .any(|slug| changed_sections.contains(slug))
            || callback.is_some_and(|value| {
                value
                    .backlinks
                    .iter()
                    .any(|slug| changed_sections.contains(slug))
            })
        {
            affected.insert(slug);
        }
    }

    // Nested transparent backlinks are rendered in the embedding page too.
    let mut parents: HashMap<Slug, Vec<Slug>> = HashMap::new();
    for (&owner, relations) in &state.relations {
        for event in &relations.events {
            if let CompileEvent::Embed(child) = event {
                parents.entry(*child).or_default().push(owner);
            }
        }
    }
    let mut pending: Vec<Slug> = affected.iter().copied().collect();
    while let Some(slug) = pending.pop() {
        if let Some(users) = parents.get(&slug) {
            for &user in users {
                if affected.insert(user) {
                    pending.push(user);
                }
            }
        }
    }
    affected.retain(|slug| state.compiled.contains_key(slug));
    affected
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::{
        section::{EmbedContent, LocalLink, SectionOption},
        writer::Writer,
    };

    fn section(slug: &str, content: HTMLContent) -> UnresolvedSection {
        let mut metadata = HTMLMetaData::with_slug_ext(Slug::new(slug), "md");
        metadata.title = Some(HTMLContent::Plain(slug.to_string()));
        metadata.builtin.page_title = Some(slug.to_string());
        UnresolvedSection { metadata, content }
    }

    fn plain(slug: &str) -> UnresolvedSection {
        section(slug, HTMLContent::Plain(format!("<p>{slug}</p>")))
    }

    fn links(target: &str) -> HTMLContent {
        HTMLContent::Lazy(vec![LazyContent::Local(LocalLink {
            url: format!("/{target}.md"),
            text: None,
        })])
    }

    fn embeds(target: &str) -> HTMLContent {
        HTMLContent::Lazy(vec![LazyContent::Embed(EmbedContent {
            url: format!("/{target}.md"),
            title: None,
            option: SectionOption::default(),
        })])
    }

    fn slugs(values: &[&str]) -> HashSet<Slug> {
        values.iter().map(Slug::new).collect()
    }

    fn assert_matches_full(state: &CompileState, shallows: &UnresolvedSections) {
        let full = compile_all_without_missing_index_warning(shallows).unwrap();
        assert_eq!(state.callback.0, full.callback.0);
        assert_eq!(state.compiled.len(), full.compiled.len());
        for (&slug, expected) in &full.compiled {
            let actual = state.compiled.get(&slug).unwrap();
            assert_eq!(
                actual.references, expected.references,
                "references in {slug}"
            );
            assert_eq!(
                Writer::html_doc(actual, state).unwrap(),
                Writer::html_doc(expected, &full).unwrap(),
                "HTML in {slug}"
            );
        }
    }

    fn with_environment(f: impl FnOnce()) {
        let root = crate::test_io::case_dir("compile-cache");
        std::fs::create_dir_all(&root).unwrap();
        environment::with_test_environment(root.clone(), environment::BuildMode::Publish, f);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn unchanged_sections_reuse_their_content_allocations() {
        with_environment(|| {
            let mut shallows = UnresolvedSections::new();
            shallows.insert(Slug::new("index"), plain("index"));
            for index in 0..128 {
                let slug = format!("page-{index}");
                shallows.insert(Slug::new(&slug), plain(&slug));
            }
            let (state, _) = CompileCache::default()
                .compile(&shallows, &HashSet::new())
                .unwrap();
            assert_eq!(state.recompiled.len(), 129);
            let SectionContent::Plain(content) =
                &state.compiled[&Slug::new("page-127")].children[0]
            else {
                panic!("expected plain content");
            };
            let original = content.as_ptr();

            shallows.get_mut(&Slug::new("page-0")).unwrap().content =
                HTMLContent::Plain("<p>changed</p>".to_string());
            let (state, affected) = state
                .into_cache()
                .compile(&shallows, &slugs(&["page-0"]))
                .unwrap();
            assert_eq!(state.recompiled, slugs(&["page-0"]));
            assert_eq!(affected, slugs(&["page-0"]));
            let SectionContent::Plain(content) =
                &state.compiled[&Slug::new("page-127")].children[0]
            else {
                panic!("expected plain content");
            };
            assert_eq!(content.as_ptr(), original);
            assert_matches_full(&state, &shallows);

            let (state, affected) = state
                .into_cache()
                .compile(&shallows, &HashSet::new())
                .unwrap();
            assert!(state.recompiled.is_empty());
            assert!(affected.is_empty());
            assert_matches_full(&state, &shallows);
        });
    }

    #[test]
    fn root_content_changes_preserve_other_headers_but_title_changes_refresh_them() {
        with_environment(|| {
            let mut shallows = UnresolvedSections::from([
                (Slug::new("index"), plain("index")),
                (Slug::new("a"), plain("a")),
                (Slug::new("b"), plain("b")),
            ]);
            let (state, _) = CompileCache::default()
                .compile(&shallows, &HashSet::new())
                .unwrap();
            shallows.get_mut(&Slug::new("index")).unwrap().content =
                HTMLContent::Plain("Changed body".to_string());
            let (state, affected) = state
                .into_cache()
                .compile(&shallows, &slugs(&["index"]))
                .unwrap();
            assert_eq!(state.recompiled, slugs(&["index"]));
            assert_eq!(affected, slugs(&["index"]));
            assert_matches_full(&state, &shallows);

            shallows
                .get_mut(&Slug::new("index"))
                .unwrap()
                .metadata
                .title = Some(HTMLContent::Plain("New root title".to_string()));
            let (state, affected) = state
                .into_cache()
                .compile(&shallows, &slugs(&["index"]))
                .unwrap();
            assert_eq!(state.recompiled, slugs(&["index"]));
            assert_eq!(affected, slugs(&["index", "a", "b"]));
            assert_matches_full(&state, &shallows);
        });
    }

    #[test]
    fn title_changes_refresh_links_metadata_and_nested_embeds_without_backlinks() {
        with_environment(|| {
            let mut shallows = UnresolvedSections::from([
                (Slug::new("index"), section("index", embeds("branch"))),
                (Slug::new("branch"), section("branch", embeds("leaf"))),
                (Slug::new("leaf"), section("leaf", links("target"))),
                (Slug::new("target"), plain("target")),
                (Slug::new("author"), plain("author")),
                (Slug::new("other"), plain("other")),
            ]);
            let target = shallows.get_mut(&Slug::new("target")).unwrap();
            target.metadata.builtin.backlinks = false;
            target.metadata.builtin.asref = Some(true);
            shallows
                .get_mut(&Slug::new("author"))
                .unwrap()
                .metadata
                .custom
                .insert("author".to_string(), links("target"));
            let (state, _) = CompileCache::default()
                .compile(&shallows, &HashSet::new())
                .unwrap();

            shallows
                .get_mut(&Slug::new("target"))
                .unwrap()
                .metadata
                .title = Some(HTMLContent::Plain("New target title".to_string()));
            let (state, affected) = state
                .into_cache()
                .compile(&shallows, &slugs(&["target"]))
                .unwrap();
            assert_eq!(
                state.recompiled,
                slugs(&["index", "branch", "leaf", "target", "author"])
            );
            assert!(!affected.contains(&Slug::new("other")));
            assert_matches_full(&state, &shallows);
        });
    }

    #[test]
    fn removed_and_added_links_refresh_both_backlink_targets() {
        with_environment(|| {
            let mut shallows = UnresolvedSections::from([
                (Slug::new("index"), plain("index")),
                (Slug::new("linker"), section("linker", links("old"))),
                (Slug::new("old"), plain("old")),
                (Slug::new("new"), plain("new")),
                (Slug::new("other"), plain("other")),
            ]);
            let (state, _) = CompileCache::default()
                .compile(&shallows, &HashSet::new())
                .unwrap();
            shallows.get_mut(&Slug::new("linker")).unwrap().content = links("new");
            let (state, affected) = state
                .into_cache()
                .compile(&shallows, &slugs(&["linker"]))
                .unwrap();
            assert_eq!(state.recompiled, slugs(&["linker"]));
            assert_eq!(affected, slugs(&["linker", "old", "new"]));
            assert_matches_full(&state, &shallows);
        });
    }

    #[test]
    fn creating_and_deleting_link_targets_invalidates_existing_links() {
        with_environment(|| {
            let mut shallows = UnresolvedSections::from([
                (Slug::new("index"), plain("index")),
                (Slug::new("linker"), section("linker", links("target"))),
                (Slug::new("other"), plain("other")),
            ]);
            let (state, _) = CompileCache::default()
                .compile(&shallows, &HashSet::new())
                .unwrap();
            shallows.insert(Slug::new("target"), plain("target"));
            let (state, affected) = state
                .into_cache()
                .compile(&shallows, &slugs(&["target"]))
                .unwrap();
            assert_eq!(state.recompiled, slugs(&["target", "linker"]));
            assert_eq!(affected, slugs(&["target", "linker"]));
            assert_matches_full(&state, &shallows);

            shallows.remove(&Slug::new("target"));
            let (state, affected) = state
                .into_cache()
                .compile(&shallows, &slugs(&["target"]))
                .unwrap();
            assert_eq!(state.recompiled, slugs(&["linker"]));
            assert_eq!(affected, slugs(&["linker"]));
            assert_matches_full(&state, &shallows);
        });
    }

    #[test]
    fn cached_embeds_preserve_parent_selection_order() {
        with_environment(|| {
            let mut shallows = UnresolvedSections::from([
                (Slug::new("index"), section("index", embeds("b"))),
                (Slug::new("a"), section("a", embeds("child"))),
                (Slug::new("b"), section("b", embeds("child"))),
                (Slug::new("child"), plain("child")),
            ]);
            let (state, _) = CompileCache::default()
                .compile(&shallows, &HashSet::new())
                .unwrap();
            assert_eq!(state.callback.0[&Slug::new("child")].parent, Slug::new("b"));
            let (state, _) = state
                .into_cache()
                .compile(&shallows, &HashSet::new())
                .unwrap();
            assert!(state.recompiled.is_empty());
            assert_matches_full(&state, &shallows);

            shallows.insert(Slug::new("index"), plain("index"));
            let (state, affected) = state
                .into_cache()
                .compile(&shallows, &slugs(&["index"]))
                .unwrap();
            assert_eq!(state.recompiled, slugs(&["index"]));
            assert!(affected.contains(&Slug::new("child")));
            assert_eq!(state.callback.0[&Slug::new("child")].parent, Slug::new("a"));
            assert_matches_full(&state, &shallows);
        });
    }

    #[test]
    fn anonymous_sections_keep_their_raw_references_and_parent_chains() {
        with_environment(|| {
            let mut shallows = UnresolvedSections::from([
                (Slug::new("index"), section("index", embeds("anonymous"))),
                (
                    Slug::new("anonymous"),
                    section(
                        "anonymous",
                        HTMLContent::Lazy(vec![
                            LazyContent::Embed(EmbedContent {
                                url: "/leaf.md".to_string(),
                                title: None,
                                option: SectionOption::default(),
                            }),
                            LazyContent::Local(LocalLink {
                                url: "/hidden.md".to_string(),
                                text: None,
                            }),
                        ]),
                    ),
                ),
                (Slug::new("leaf"), section("leaf", links("target"))),
                (Slug::new("hidden"), plain("hidden")),
                (Slug::new("target"), plain("target")),
                (Slug::new("other"), plain("other")),
            ]);
            shallows
                .get_mut(&Slug::new("anonymous"))
                .unwrap()
                .metadata
                .builtin
                .internal_anon_subtree = true;
            let hidden = shallows.get_mut(&Slug::new("hidden")).unwrap();
            hidden.metadata.builtin.internal_anon_subtree = true;
            hidden.metadata.builtin.asref = Some(true);
            shallows
                .get_mut(&Slug::new("target"))
                .unwrap()
                .metadata
                .builtin
                .asref = Some(true);
            let (state, _) = CompileCache::default()
                .compile(&shallows, &HashSet::new())
                .unwrap();
            let (state, _) = state
                .into_cache()
                .compile(&shallows, &HashSet::new())
                .unwrap();
            assert!(state.recompiled.is_empty());
            assert_matches_full(&state, &shallows);

            shallows.get_mut(&Slug::new("leaf")).unwrap().content =
                HTMLContent::Plain("Changed leaf".to_string());
            let (state, _) = state
                .into_cache()
                .compile(&shallows, &slugs(&["leaf"]))
                .unwrap();
            assert!(!state.recompiled.contains(&Slug::new("other")));
            assert_matches_full(&state, &shallows);

            shallows
                .get_mut(&Slug::new("hidden"))
                .unwrap()
                .metadata
                .builtin
                .internal_anon_subtree = false;
            let (state, _) = state
                .into_cache()
                .compile(&shallows, &slugs(&["hidden"]))
                .unwrap();
            assert_matches_full(&state, &shallows);
        });
    }

    #[test]
    fn backlink_content_changes_update_transparent_embeds_without_recompiling_them() {
        with_environment(|| {
            let mut shallows = UnresolvedSections::from([
                (Slug::new("index"), section("index", embeds("branch"))),
                (Slug::new("branch"), plain("branch")),
                (Slug::new("linker"), section("linker", links("branch"))),
                (Slug::new("other"), plain("other")),
            ]);
            let branch = shallows.get_mut(&Slug::new("branch")).unwrap();
            branch.metadata.builtin.transparent_backlinks = true;
            branch.metadata.builtin.footer_mode = Some(FooterMode::Embed);
            let (state, _) = CompileCache::default()
                .compile(&shallows, &HashSet::new())
                .unwrap();
            let HTMLContent::Lazy(contents) =
                &mut shallows.get_mut(&Slug::new("linker")).unwrap().content
            else {
                unreachable!()
            };
            contents.push(LazyContent::Plain("Changed backlink content".to_string()));
            let (state, affected) = state
                .into_cache()
                .compile(&shallows, &slugs(&["linker"]))
                .unwrap();
            assert_eq!(state.recompiled, slugs(&["linker"]));
            assert_eq!(affected, slugs(&["index", "branch", "linker"]));
            assert_matches_full(&state, &shallows);
        });
    }

    #[test]
    fn successive_graph_changes_match_full_compilation_and_output_selection() {
        with_environment(|| {
            let names = [
                "index", "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k",
            ];
            let mut rng = fastrand::Rng::with_seed(747_571);
            let mut shallows: UnresolvedSections = names
                .iter()
                .map(|name| (Slug::new(name), plain(name)))
                .collect();
            let (mut state, _) = CompileCache::default()
                .compile(&shallows, &HashSet::new())
                .unwrap();
            for revision in 0..48 {
                let before: HashMap<Slug, String> = state
                    .compiled
                    .iter()
                    .map(|(&slug, section)| (slug, Writer::html_doc(section, &state).unwrap().0))
                    .collect();
                let owner = rng.usize(..names.len());
                let slug = names[owner];
                let target = names[rng.usize(..names.len())];
                let mut contents = vec![
                    LazyContent::Plain(format!("Revision {revision}")),
                    LazyContent::Local(LocalLink {
                        url: format!("/{target}.md"),
                        text: None,
                    }),
                ];
                if owner + 1 < names.len() && rng.bool() {
                    let child = names[rng.usize(owner + 1..names.len())];
                    contents.push(LazyContent::Embed(EmbedContent {
                        url: format!("/{child}.md"),
                        title: None,
                        option: SectionOption::new(rng.bool(), rng.bool(), rng.bool()),
                    }));
                }
                let mut updated = section(slug, HTMLContent::Lazy(contents));
                updated.metadata.title = Some(if rng.bool() {
                    links(target)
                } else {
                    HTMLContent::Plain(format!("Title {revision}"))
                });
                updated.metadata.builtin.asref = Some(rng.bool());
                updated.metadata.builtin.asback = Some(rng.bool());
                updated.metadata.builtin.backlinks = rng.bool();
                updated.metadata.builtin.transparent_backlinks = rng.bool();
                updated.metadata.builtin.internal_anon_subtree = owner != 0 && rng.bool();
                updated.metadata.builtin.footer_mode = Some(if rng.bool() {
                    FooterMode::Embed
                } else {
                    FooterMode::Link
                });
                if rng.bool() {
                    updated.metadata.builtin.parent = Some(Slug::new("index"));
                    updated
                        .metadata
                        .custom
                        .insert("author".to_string(), links(target));
                }
                shallows.insert(Slug::new(slug), updated);
                let (next, affected) = state
                    .into_cache()
                    .compile(&shallows, &slugs(&[slug]))
                    .unwrap();
                assert_matches_full(&next, &shallows);
                for (&slug, section) in &next.compiled {
                    let html = Writer::html_doc(section, &next).unwrap().0;
                    if before.get(&slug) != Some(&html) {
                        assert!(
                            affected.contains(&slug),
                            "changed page {slug} was not selected at revision {revision}"
                        );
                    }
                }
                state = next;
            }
        });
    }

    #[test]
    fn changed_embeds_still_report_cycles() {
        with_environment(|| {
            let mut shallows = UnresolvedSections::from([
                (Slug::new("index"), section("index", embeds("a"))),
                (Slug::new("a"), plain("a")),
                (Slug::new("b"), section("b", embeds("a"))),
            ]);
            let (state, _) = CompileCache::default()
                .compile(&shallows, &HashSet::new())
                .unwrap();
            shallows.get_mut(&Slug::new("a")).unwrap().content = embeds("b");
            let error = state
                .into_cache()
                .compile(&shallows, &slugs(&["a"]))
                .unwrap_err();
            assert!(error.to_string().contains("cyclic embed detected"));
        });
    }
}
