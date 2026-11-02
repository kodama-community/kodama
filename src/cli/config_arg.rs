// Copyright (c) 2025 Kodama Project. All rights reserved.
// Released under the GPL-3.0 license as described in the file LICENSE.
// Authors: Kokic (@kokic)

use camino::Utf8PathBuf;
use eyre::eyre;

use crate::config::{self, ConfigSource};

/// Choose a configured site or an in-place Free workspace.
pub fn resolve(config: Option<&Utf8PathBuf>, free: bool) -> eyre::Result<ConfigSource> {
    if free {
        let root = current_dir()?;
        if root.join(config::DEFAULT_CONFIG_PATH).is_file() {
            color_print::ceprintln!(
                "<dim>[config] Ignoring `{}` because `--free` was given.</>",
                config::DEFAULT_CONFIG_PATH
            );
        }
        return Ok(ConfigSource::free(root));
    }

    Ok(ConfigSource::file(
        config
            .cloned()
            .unwrap_or_else(|| config::DEFAULT_CONFIG_PATH.into()),
    ))
}

fn current_dir() -> eyre::Result<Utf8PathBuf> {
    let cwd = std::env::current_dir().map_err(|err| eyre!("cannot resolve current dir: {err}"))?;
    Utf8PathBuf::from_path_buf(cwd)
        .map_err(|path| eyre!("current dir is not valid UTF-8: {}", path.display()))
}
