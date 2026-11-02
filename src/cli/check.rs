// Copyright (c) 2025 Kodama Project. All rights reserved.
// Released under the GPL-3.0 license as described in the file LICENSE.
// Authors: Kokic (@kokic), Spore (@s-cerevisiae)

use std::collections::{HashMap, HashSet};

use camino::Utf8PathBuf;
use eyre::{bail, eyre, WrapErr};

use crate::{
    cli::config_arg,
    compiler::{
        self, inline_typst,
        section::{HTMLContent, LazyContent, UnresolvedSection},
        Diagnostic, Severity,
    },
    environment::{self, BuildMode},
    path_utils,
    slug::{self, SectionKind, Slug},
};

#[derive(clap::Args)]
pub struct CheckCommand {
    /// Path to the configuration file (default: ./Kodama.toml).
    #[arg(short, long, conflicts_with = "free")]
    config: Option<Utf8PathBuf>,

    /// Check documents under the current directory without reading config.
    #[arg(long)]
    free: bool,

    /// Treat warnings as errors.
    #[arg(long, default_value_t = false)]
    strict: bool,
}

pub fn check(command: &CheckCommand) -> eyre::Result<()> {
    environment::init_environment(
        &config_arg::resolve(command.config.as_ref(), command.free)?,
        BuildMode::Check,
    )?;
    crate::typst::cache::begin_build();

    let trees_dir = environment::trees_dir();
    let workspace = compiler::all_trees_source(trees_dir.as_path())
        .wrap_err_with(|| eyre!("failed to scan trees dir `{}`", trees_dir))?;

    let mut diagnostics = Vec::new();
    if workspace.slug_exts.is_empty() {
        diagnostics.push(Diagnostic::hint(format!(
            "No sections found under `{}`.",
            trees_dir
        )));
    }
    if !environment::is_free() && !workspace.slug_exts.contains_key(&Slug::new("index")) {
        diagnostics.push(Diagnostic::warning(format!(
            "Missing `index` section. Add `index.{}` or `index.{}`.",
            environment::markdown_suffix(),
            environment::typst_suffix(),
        )));
    }

    let shallows = parse_shallows_no_cache(&workspace, &mut diagnostics);
    collect_dangling_local_links(&shallows, &mut diagnostics);
    let has_parse_errors = diagnostics.iter().any(|d| d.severity.is_error());
    if !has_parse_errors {
        validate_compile_graph(&shallows, &mut diagnostics);
    }

    for diagnostic in &diagnostics {
        print_diagnostic(diagnostic);
    }

    let errors = diagnostics.iter().filter(|d| d.severity.is_error()).count();
    let warnings = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Warning)
        .count();
    let hints = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Hint)
        .count();

    let strict_note = if command.strict { " (strict mode)" } else { "" };
    println!(
        "Check result: {} error(s), {} warning(s), {} hint(s){}.",
        errors, warnings, hints, strict_note
    );

    if errors > 0 {
        bail!("check failed with {} error(s)", errors);
    }
    if command.strict && warnings > 0 {
        bail!("check failed in strict mode with {} warning(s)", warnings);
    }
    Ok(())
}

fn parse_shallows_no_cache(
    workspace: &compiler::Workspace,
    diagnostics: &mut Vec<Diagnostic>,
) -> HashMap<Slug, UnresolvedSection> {
    let mut shallows = HashMap::new();
    let mut entries: Vec<(Slug, SectionKind)> = workspace
        .slug_exts
        .iter()
        .map(|(&slug, &kind)| (slug, kind))
        .collect();
    entries.sort_by_key(|(slug, _)| slug.as_str());

    inline_typst::begin_inline_batch();
    for (slug, kind) in entries {
        let suffix = environment::suffix_for_kind(kind);
        match compiler::parse_source_sections(slug, kind) {
            Ok(output) => {
                // Rendering/read failures count as errors here, including include
                // failures that ordinary builds report as warnings and render in place.
                diagnostics.extend(
                    output
                        .diagnostics
                        .into_iter()
                        .map(|diagnostic| Diagnostic::error(diagnostic.message)),
                );
                for (section_slug, section) in output.sections {
                    if shallows.insert(section_slug, section).is_some() {
                        diagnostics.push(Diagnostic::error(format!(
                            "Duplicate section slug `{section_slug}` generated while parsing `{slug}.{suffix}`."
                        )));
                    }
                }
            }
            Err(err) => diagnostics.push(Diagnostic::error(format!(
                "Failed to parse `{slug}.{suffix}`: {err:#}"
            ))),
        }
    }

    // Compile inline formulas (reporting errors) and resolve placeholders.
    let results = match inline_typst::compile_inline_batch() {
        Ok(batch) => {
            diagnostics.extend(batch.diagnostics);
            batch.contents
        }
        Err(err) => {
            diagnostics.push(Diagnostic::error(format!("{err:#}")));
            Vec::new()
        }
    };
    for section in shallows.values_mut() {
        inline_typst::resolve_inline_typst_section(section, &results);
    }

    shallows
}

fn collect_dangling_local_links(
    shallows: &HashMap<Slug, UnresolvedSection>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut seen = HashSet::new();
    for (&from_slug, section) in shallows {
        let HTMLContent::Lazy(contents) = &section.content else {
            continue;
        };
        for content in contents {
            let LazyContent::Local(local) = content else {
                continue;
            };
            let target_slug = resolve_subsection_slug(from_slug, &local.url);
            if shallows.contains_key(&target_slug) {
                continue;
            }
            if seen.insert((from_slug, target_slug, local.url.clone())) {
                diagnostics.push(Diagnostic::warning(format!(
                    "Dangling local link in `{}`: `{}` resolves to missing section `{}`.",
                    from_slug, local.url, target_slug
                )));
            }
        }
    }
}

fn validate_compile_graph(
    shallows: &HashMap<Slug, UnresolvedSection>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if shallows.is_empty() {
        return;
    }
    if let Err(err) = compiler::state::compile_all_without_missing_index_warning(shallows) {
        diagnostics.push(Diagnostic::error(format!(
            "Failed to compile section graph: {err:#}"
        )));
    }
}

fn resolve_subsection_slug(current_slug: Slug, url: &str) -> Slug {
    slug::to_slug(path_utils::relative_to_current(current_slug.as_str(), url))
}

fn print_diagnostic(diagnostic: &Diagnostic) {
    match diagnostic.severity {
        Severity::Fatal | Severity::Error => {
            color_print::ceprintln!("<r>Error:</> {}", diagnostic.message)
        }
        Severity::Warning => color_print::ceprintln!("<y>Warning:</> {}", diagnostic.message),
        Severity::Hint => color_print::ceprintln!("<dim>Hint:</> {}", diagnostic.message),
    }
}
