// Copyright (c) 2025 Kodama Project. All rights reserved.
// Released under the GPL-3.0 license as described in the file LICENSE.

use std::{
    cell::RefCell,
    collections::BTreeMap,
    fs::{self, OpenOptions},
    process::Command,
};

use camino::{Utf8Component, Utf8Path, Utf8PathBuf};
use eyre::{bail, eyre, WrapErr};
use serde::{Deserialize, Serialize};

use crate::environment::{self, SourceMeta};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Dependencies {
    direct: DependencySnapshot,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    inline: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct DependencySnapshot {
    version: Option<String>,
    files: BTreeMap<String, SourceMeta>,
}

impl DependencySnapshot {
    fn is_current(&self) -> eyre::Result<bool> {
        let Some(version) = &self.version else {
            return Ok(self.files.is_empty());
        };
        Ok(version == &typst_version()?
            && self.files.iter().all(|(path, meta)| {
                environment::source_meta_of(Utf8Path::new(path)) == Some(*meta)
            }))
    }

    fn merge(&mut self, other: &Self) {
        if other.version.is_some() {
            self.version.clone_from(&other.version);
        }
        self.files
            .extend(other.files.iter().map(|(path, meta)| (path.clone(), *meta)));
    }
}

impl Dependencies {
    pub fn is_current(&self) -> eyre::Result<bool> {
        if !self.direct.is_current()? {
            return Ok(false);
        }
        match &self.inline {
            Some(generation) => INLINE_CACHE.with(|cache| {
                let mut cache = cache.borrow_mut();
                let state = inline_cache_state(&mut cache)?;
                Ok(state.current
                    && state
                        .record
                        .as_ref()
                        .is_some_and(|record| &record.generation == generation))
            }),
            None => Ok(true),
        }
    }

    pub fn merge(&mut self, other: &Self) {
        self.direct.merge(&other.direct);
        if other.inline.is_some() {
            self.inline.clone_from(&other.inline);
        }
    }

    fn merge_owned(&mut self, other: Self) {
        if other.direct.version.is_some() {
            self.direct.version = other.direct.version;
        }
        self.direct.files.extend(other.direct.files);
        if other.inline.is_some() {
            self.inline = other.inline;
        }
    }
}

#[derive(Serialize, Deserialize)]
struct InlineCache {
    generation: String,
    dependencies: DependencySnapshot,
}

struct InlineCacheState {
    record: Option<InlineCache>,
    current: bool,
}

thread_local! {
    static VERSION: RefCell<Option<String>> = const { RefCell::new(None) };
    static DEPENDENCIES: RefCell<Dependencies> = RefCell::new(Dependencies::default());
    static INLINE_CACHE: RefCell<Option<InlineCacheState>> = const { RefCell::new(None) };
}

pub fn begin_build() {
    VERSION.with(|version| *version.borrow_mut() = None);
    INLINE_CACHE.with(|cache| *cache.borrow_mut() = None);
    begin_phase();
}

pub fn begin_phase() {
    DEPENDENCIES.with(|dependencies| *dependencies.borrow_mut() = Dependencies::default());
}

pub fn record_dependencies(dependencies: Dependencies) {
    DEPENDENCIES.with(|current| current.borrow_mut().merge_owned(dependencies));
}

pub fn capture_dependencies<T>(
    render: impl FnOnce() -> eyre::Result<T>,
) -> eyre::Result<(T, Dependencies)> {
    let previous = DEPENDENCIES.with(|current| std::mem::take(&mut *current.borrow_mut()));
    let result = render();
    let captured = DEPENDENCIES.with(|current| current.replace(previous));
    result.map(|result| (result, captured))
}

fn inline_cache_path() -> Utf8PathBuf {
    environment::hash_dir().join("typst/inline.dependencies")
}

fn inline_cache_state(cache: &mut Option<InlineCacheState>) -> eyre::Result<&mut InlineCacheState> {
    if cache.is_none() {
        let record = fs::read(inline_cache_path())
            .ok()
            .and_then(|bytes| serde_json::from_slice::<InlineCache>(&bytes).ok());
        let current = match &record {
            Some(record) => record.dependencies.is_current()?,
            None => false,
        };
        *cache = Some(InlineCacheState { record, current });
    }
    Ok(cache.as_mut().unwrap())
}

/// Keep one shared report rather than copying the batch's file list into every page.
pub fn share_inline_dependencies(dependencies: Dependencies) -> eyre::Result<Dependencies> {
    if environment::is_check() {
        return Ok(dependencies);
    }
    debug_assert!(dependencies.inline.is_none());
    INLINE_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        let state = inline_cache_state(&mut cache)?;
        let reuse = state.current && !*crate::cli::build::no_cache_enabled();
        if reuse {
            let record = state.record.as_ref().unwrap();
            if record.dependencies.version == dependencies.direct.version
                && dependencies
                    .direct
                    .files
                    .iter()
                    .all(|(path, meta)| record.dependencies.files.get(path) == Some(meta))
            {
                return Ok(Dependencies {
                    inline: Some(record.generation.clone()),
                    ..Default::default()
                });
            }
        }
        let record = if reuse {
            // Retain dependencies of pages not reparsed during this build.
            let mut record = state.record.take().unwrap();
            let conflicting = record.dependencies.version != dependencies.direct.version
                || dependencies.direct.files.iter().any(|(path, meta)| {
                    record
                        .dependencies
                        .files
                        .get(path)
                        .is_some_and(|previous| previous != meta)
                });
            if conflicting {
                record.generation = new_inline_generation();
            }
            record.dependencies.version = dependencies.direct.version;
            record.dependencies.files.extend(dependencies.direct.files);
            record
        } else {
            // Old page entries must remain stale if writing their replacements fails.
            InlineCache {
                generation: new_inline_generation(),
                dependencies: dependencies.direct,
            }
        };
        let path = inline_cache_path();
        environment::create_parent_dirs(&path);
        state.current = false;
        fs::write(path, serde_json::to_vec(&record)?)
            .wrap_err("failed to write inline Typst dependency cache")?;
        let generation = record.generation.clone();
        state.record = Some(record);
        state.current = true;
        Ok(Dependencies {
            inline: Some(generation),
            ..Default::default()
        })
    })
}

fn new_inline_generation() -> String {
    format!("{:016x}{:016x}", fastrand::u64(..), fastrand::u64(..))
}

fn typst_version() -> eyre::Result<String> {
    if let Some(version) = VERSION.with(|version| version.borrow().clone()) {
        return Ok(version);
    }
    let output = Command::new("typst")
        .arg("--version")
        .output()
        .wrap_err("failed to run Typst; install Typst and put it on PATH")?;
    if !output.status.success() {
        bail!(
            "failed to read Typst version: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let version = String::from_utf8(output.stdout)?.trim().to_string();
    VERSION.with(|current| *current.borrow_mut() = Some(version.clone()));
    Ok(version)
}

/// A per-invocation dependency report. Check mode only writes this temporary
/// file, never a site cache or published output.
pub struct DependencyFile {
    path: Utf8PathBuf,
    from_stdin: bool,
}

impl DependencyFile {
    pub fn new(command: &mut Command, from_stdin: bool) -> eyre::Result<Self> {
        typst_version()?;
        let path = std::env::temp_dir().join(format!(
            "kodama-typst-deps-{}-{:x}.json",
            std::process::id(),
            fastrand::u64(..)
        ));
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        let path = Utf8PathBuf::from_path_buf(path)
            .map_err(|_| eyre!("Typst dependency report path is not UTF-8"))?;
        command.arg("--deps").arg(&path).arg("--deps-format=json");
        Ok(Self { path, from_stdin })
    }

    pub fn read(&self) -> eyre::Result<Dependencies> {
        #[derive(Deserialize)]
        struct Report {
            inputs: Vec<String>,
        }
        let report: Report = serde_json::from_slice(&fs::read(&self.path)?)
            .wrap_err("failed to read Typst dependency report")?;
        let mut direct = DependencySnapshot {
            version: Some(typst_version()?),
            files: BTreeMap::new(),
        };
        for path in report.inputs {
            if self.from_stdin && Utf8Path::new(&path).file_name() == Some("<stdin>") {
                continue;
            }
            let path = Utf8PathBuf::from_path_buf(
                fs::canonicalize(&path)
                    .wrap_err_with(|| eyre!("failed to read Typst dependency `{path}`"))?,
            )
            .map_err(|_| eyre!("Typst dependency path is not UTF-8"))?;
            let meta = environment::source_meta_of(&path)
                .ok_or_else(|| eyre!("failed to stat Typst dependency `{path}`"))?;
            direct.files.insert(path.into_string(), meta);
        }
        Ok(Dependencies {
            direct,
            inline: None,
        })
    }
}

impl Drop for DependencyFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[derive(Serialize, Deserialize)]
struct ArtifactCache<D> {
    source: String,
    output: String,
    dependencies: D,
}

fn artifact_cache_path(input: &Utf8Path, format: &str) -> eyre::Result<Utf8PathBuf> {
    let mut relative = Utf8PathBuf::from("typst").join(format);
    for component in input.components() {
        match component {
            Utf8Component::Normal(part) => relative.push(part),
            Utf8Component::CurDir => {}
            _ => bail!("Typst cache input must be a source-tree-relative path: `{input}`"),
        }
    }
    Ok(environment::hash_file_path(relative))
}

pub fn cached_artifact_dependencies(
    input: &Utf8Path,
    output: &Utf8Path,
    format: &str,
) -> eyre::Result<Option<Dependencies>> {
    if *crate::cli::build::no_cache_enabled() || !output.exists() {
        return Ok(None);
    }
    let path = artifact_cache_path(input, format)?;
    let cached = fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<ArtifactCache<Dependencies>>(&bytes).ok());
    if let Some(cached) = cached {
        if cached.source == environment::trees_dir().join(input).as_str()
            && cached.output == output.as_str()
            && cached.dependencies.is_current()?
        {
            return Ok(Some(cached.dependencies));
        }
    }
    Ok(None)
}

pub fn save_artifact(
    input: &Utf8Path,
    output: &Utf8Path,
    format: &str,
    dependencies: &Dependencies,
) -> eyre::Result<()> {
    let path = artifact_cache_path(input, format)?;
    let bytes = serde_json::to_vec(&ArtifactCache {
        source: environment::trees_dir().join(input).to_string(),
        output: output.to_string(),
        dependencies,
    })?;
    fs::write(path, bytes).wrap_err("failed to write Typst artifact cache")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dependency(path: &str) -> Dependencies {
        Dependencies {
            direct: DependencySnapshot {
                files: BTreeMap::from([(
                    path.to_string(),
                    SourceMeta {
                        modified_ns: 1,
                        size: 1,
                    },
                )]),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn nested_captures_keep_dependencies_in_their_own_scope() {
        begin_phase();
        let (_, outer) = capture_dependencies(|| {
            record_dependencies(dependency("outer.typ"));
            let (_, inner) = capture_dependencies(|| {
                record_dependencies(dependency("inner.typ"));
                Ok(())
            })?;
            assert_eq!(
                inner
                    .direct
                    .files
                    .keys()
                    .map(String::as_str)
                    .collect::<Vec<_>>(),
                ["inner.typ"]
            );
            let (_, empty) = capture_dependencies(|| Ok(()))?;
            assert!(empty.direct.files.is_empty());
            Ok(())
        })
        .unwrap();
        assert_eq!(
            outer
                .direct
                .files
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            ["outer.typ"]
        );
        DEPENDENCIES.with(|current| assert!(current.borrow().direct.files.is_empty()));
    }

    #[test]
    fn failed_capture_restores_outer_dependencies() {
        begin_phase();
        record_dependencies(dependency("outer.typ"));
        let result: eyre::Result<((), Dependencies)> = capture_dependencies(|| {
            record_dependencies(dependency("failed.typ"));
            Err(eyre!("render failed"))
        });
        assert!(result.is_err());
        DEPENDENCIES.with(|current| {
            let current = current.borrow();
            assert_eq!(
                current
                    .direct
                    .files
                    .keys()
                    .map(String::as_str)
                    .collect::<Vec<_>>(),
                ["outer.typ"]
            );
        });
        begin_phase();
    }
}
