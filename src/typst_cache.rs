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
    version: Option<String>,
    files: BTreeMap<String, SourceMeta>,
}

impl Dependencies {
    pub fn is_current(&self) -> eyre::Result<bool> {
        let Some(version) = &self.version else {
            return Ok(self.files.is_empty());
        };
        Ok(version == &typst_version()?
            && self.files.iter().all(|(path, meta)| {
                environment::source_meta_of(Utf8Path::new(path)) == Some(*meta)
            }))
    }

    pub fn merge(&mut self, other: &Self) {
        if other.version.is_some() {
            self.version.clone_from(&other.version);
        }
        self.files.extend(other.files.clone());
    }
}

thread_local! {
    static VERSION: RefCell<Option<String>> = const { RefCell::new(None) };
    static DEPENDENCIES: RefCell<Dependencies> = RefCell::new(Dependencies::default());
}

pub fn begin_build() {
    VERSION.with(|version| *version.borrow_mut() = None);
    begin_phase();
}

pub fn begin_phase() {
    DEPENDENCIES.with(|dependencies| *dependencies.borrow_mut() = Dependencies::default());
}

pub fn current_dependencies() -> Dependencies {
    DEPENDENCIES.with(|dependencies| dependencies.borrow().clone())
}

pub fn record_dependencies(dependencies: &Dependencies) {
    DEPENDENCIES.with(|current| current.borrow_mut().merge(dependencies));
}

pub fn capture_dependencies<T>(
    render: impl FnOnce() -> eyre::Result<T>,
) -> eyre::Result<(T, Dependencies)> {
    let previous = DEPENDENCIES.with(|current| std::mem::take(&mut *current.borrow_mut()));
    let result = render();
    let captured = current_dependencies();
    record_dependencies(&previous);
    result.map(|result| (result, captured))
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
        let mut dependencies = Dependencies {
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
            dependencies.files.insert(path.into_string(), meta);
        }
        record_dependencies(&dependencies);
        Ok(dependencies)
    }
}

impl Drop for DependencyFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[derive(Serialize, Deserialize)]
struct ArtifactCache {
    source: String,
    output: String,
    dependencies: Dependencies,
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

pub fn artifact_is_current(
    input: &Utf8Path,
    output: &Utf8Path,
    format: &str,
) -> eyre::Result<bool> {
    if *crate::cli::build::no_cache_enabled() || !output.exists() {
        return Ok(false);
    }
    let path = artifact_cache_path(input, format)?;
    let cached = fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<ArtifactCache>(&bytes).ok());
    if let Some(cached) = cached {
        if cached.source == environment::trees_dir().join(input).as_str()
            && cached.output == output.as_str()
            && cached.dependencies.is_current()?
        {
            record_dependencies(&cached.dependencies);
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn save_artifact(
    input: &Utf8Path,
    output: &Utf8Path,
    format: &str,
    dependencies: Dependencies,
) -> eyre::Result<()> {
    let path = artifact_cache_path(input, format)?;
    let bytes = serde_json::to_vec(&ArtifactCache {
        source: environment::trees_dir().join(input).to_string(),
        output: output.to_string(),
        dependencies,
    })?;
    fs::write(path, bytes).wrap_err("failed to write Typst artifact cache")
}
