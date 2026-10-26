// Copyright (c) 2025 Kodama Project. All rights reserved.
// Released under the GPL-3.0 license as described in the file LICENSE.
// Authors: Kokic (@kokic), Spore (@s-cerevisiae)

use std::time::UNIX_EPOCH;

use camino::Utf8Path;
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
