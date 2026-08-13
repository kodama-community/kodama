// Copyright (c) 2025 Kodama Project. All rights reserved.
// Released under the GPL-3.0 license as described in the file LICENSE.
// Authors: Kokic (@kokic), Spore (@s-cerevisiae)

use std::time::UNIX_EPOCH;

use camino::Utf8Path;
use eyre::{eyre, Context};
use serde::{Deserialize, Serialize};

/// A source file's (mtime, size) snapshot, used for change detection without
/// reading the file's content. Unlike a content hash it is cheap to obtain and
/// its value is independent of the Rust compiler version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceMeta {
    /// Modified time in nanoseconds since the Unix epoch.
    pub modified_ns: u128,
    /// File size in bytes.
    pub size: u64,
}

impl SourceMeta {
    pub fn from_metadata(metadata: &std::fs::Metadata) -> Option<SourceMeta> {
        let modified_ns = metadata
            .modified()
            .ok()?
            .duration_since(UNIX_EPOCH)
            .ok()?
            .as_nanos();
        Some(SourceMeta {
            modified_ns,
            size: metadata.len(),
        })
    }
}

/// Return the (mtime, size) snapshot of `path`, if it can be stat'ed.
pub fn source_meta_of<P: AsRef<Utf8Path>>(path: P) -> Option<SourceMeta> {
    std::fs::metadata(path.as_ref())
        .ok()
        .and_then(|metadata| SourceMeta::from_metadata(&metadata))
}

/// Return the (mtime, size) snapshot of a source file under the trees directory.
pub fn relative_source_meta<P: AsRef<Utf8Path>>(relative_path: P) -> Option<SourceMeta> {
    source_meta_of(super::trees_dir().join(relative_path.as_ref()))
}

fn read_meta_file(path: &Utf8Path) -> Option<SourceMeta> {
    let text = std::fs::read_to_string(path).ok()?;
    let (modified_ns, size) = text.trim().split_once(':')?;
    Some(SourceMeta {
        modified_ns: modified_ns.parse().ok()?,
        size: size.parse().ok()?,
    })
}

fn write_meta_file(path: &Utf8Path, meta: SourceMeta) -> eyre::Result<()> {
    std::fs::write(path, format!("{}:{}", meta.modified_ns, meta.size))
        .wrap_err_with(|| eyre!("failed to write change-detection file `{}`", path))
}

/// Returns whether `relative_path`'s (mtime, size) differs from the last
/// recorded baseline, updating the baseline in place. The baseline lives in the
/// cache "hash" directory but holds a metadata snapshot, not a content hash.
/// Used to skip regenerating outputs that derive from a source file without
/// reading the source's content (e.g. cached typst HTML/SVG).
pub fn file_meta_updated<P: AsRef<Utf8Path>>(relative_path: P) -> eyre::Result<bool> {
    if *crate::cli::build::no_cache_enabled() {
        return Ok(true);
    }

    let relative_path = relative_path.as_ref();
    let current = relative_source_meta(relative_path).ok_or_else(|| {
        eyre!(
            "failed to stat file `{}`",
            super::trees_dir().join(relative_path)
        )
    })?;
    let meta_path = super::hash_file_path(relative_path);
    let is_modified = read_meta_file(meta_path.as_path()) != Some(current);
    if is_modified {
        write_meta_file(meta_path.as_path(), current)?;
    }
    Ok(is_modified)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn test_source_meta_roundtrip_via_meta_file() {
        let root = crate::test_io::case_dir("env-meta-roundtrip");
        fs::create_dir_all(root.as_std_path()).unwrap();
        let path = root.join("meta.txt");

        let meta = SourceMeta {
            modified_ns: 1_700_000_000_000_000_123,
            size: 42,
        };
        write_meta_file(path.as_path(), meta).unwrap();
        assert_eq!(read_meta_file(path.as_path()), Some(meta));

        let _ = fs::remove_dir_all(root.as_std_path());
    }

    #[test]
    fn test_read_meta_file_rejects_legacy_content_hash() {
        let root = crate::test_io::case_dir("env-meta-legacy");
        fs::create_dir_all(root.as_std_path()).unwrap();
        let path = root.join("meta.txt");
        fs::write(path.as_std_path(), "1234567890").unwrap();

        assert_eq!(read_meta_file(path.as_path()), None);

        let _ = fs::remove_dir_all(root.as_std_path());
    }

    #[test]
    fn test_file_meta_updated_detects_changes() {
        let root = crate::test_io::case_dir("env-meta-updated");
        fs::create_dir_all(root.as_std_path()).unwrap();

        super::super::with_test_environment(root.clone(), super::super::BuildMode::Publish, || {
            let relative = "meta-tests/a.typst";
            let full_path = super::super::trees_dir().join(relative);
            fs::create_dir_all(full_path.parent().unwrap().as_std_path()).unwrap();
            fs::write(full_path.as_std_path(), "v1").unwrap();

            assert!(file_meta_updated(relative).unwrap());
            assert!(!file_meta_updated(relative).unwrap());

            fs::write(full_path.as_std_path(), "v22").unwrap();
            assert!(file_meta_updated(relative).unwrap());
        });

        let _ = fs::remove_dir_all(root.as_std_path());
    }
}
