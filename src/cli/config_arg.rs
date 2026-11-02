// Copyright (c) 2025 Kodama Project. All rights reserved.
// Released under the GPL-3.0 license as described in the file LICENSE.
// Authors: Kokic (@kokic)

use camino::Utf8PathBuf;
use eyre::eyre;

use crate::config::{self, ConfigSource};

/// Choose a configured site or an in-place Free workspace.
pub fn resolve(
    config: Option<&Utf8PathBuf>,
    free: bool,
    no_config: bool,
) -> eyre::Result<ConfigSource> {
    if no_config {
        return Err(eyre!("`--no-config` is retired. Use a configuration file, or explicitly choose `--free` to write HTML beside the source files."));
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_defaults_to_the_default_config_path() {
        let source = resolve(None, false, false).unwrap();
        match source {
            ConfigSource::File(path) => assert_eq!(path, config::DEFAULT_CONFIG_PATH),
            other => panic!("unexpected source: {other:?}"),
        }
    }

    #[test]
    fn test_resolve_keeps_the_given_config_path() {
        let path = Utf8PathBuf::from("custom/site.toml");
        let source = resolve(Some(&path), false, false).unwrap();
        match source {
            ConfigSource::File(resolved) => assert_eq!(resolved, path),
            other => panic!("unexpected source: {other:?}"),
        }
    }

    #[test]
    fn test_resolve_free_uses_the_current_directory() {
        let source = resolve(None, true, false).unwrap();
        match source {
            ConfigSource::Free(root) => assert_eq!(root, current_dir().unwrap()),
            other => panic!("unexpected source: {other:?}"),
        }
    }
}
