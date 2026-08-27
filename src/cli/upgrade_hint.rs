// Copyright (c) 2025 Kodama Project. All rights reserved.
// Released under the GPL-3.0 license as described in the file LICENSE.
// Authors: Kokic (@kokic)

use camino::{Utf8Path, Utf8PathBuf};
use eyre::WrapErr;

use crate::{cli::upgrade::upgrade_content, environment, path_utils};

/// Best-effort check for files that may need an explicit `kodama upgrade`.
///
/// This only prints a hint and never writes to user files. Each file is
/// tracked by its (mtime, size) snapshot stored in the cache, so an
/// out-of-date file is hinted at most once per distinct file state, and
/// unchanged files skip the content comparison entirely.
pub fn check_upgrade_hints() {
    if let Err(err) = check_upgrade_hints_inner() {
        color_print::ceprintln!(
            "<dim>[upgrade] Warning: failed to check for outdated files: {err:#}</>"
        );
    }
}

fn check_upgrade_hints_inner() -> eyre::Result<()> {
    check_config_hint()?;
    check_kodama_typ_hint()?;
    Ok(())
}

fn check_config_hint() -> eyre::Result<()> {
    let path = environment::config_file();
    if check_outdated_with(path.as_path(), "config", is_config_up_to_date)? {
        color_print::ceprintln!(
            "<y>[upgrade] Hint: config file `{}` is missing configuration introduced by this version.\n<y>  Run `kodama upgrade config` to update it.</>",
            path_utils::pretty_path(path.as_path())
        );
    }
    Ok(())
}

fn check_kodama_typ_hint() -> eyre::Result<()> {
    let path = environment::trees_dir().join("_lib").join("kodama.typ");
    let bundled = include_str!("../include/kodama.typ");
    if check_outdated_with(path.as_path(), "kodama-typ", |content| content == bundled)? {
        color_print::ceprintln!(
            "<y>[upgrade] Hint: Typst library `{}` differs from the version bundled with Kodama.\n<y>  Run `kodama upgrade typst-lib` to sync it.</>",
            path_utils::pretty_path(path.as_path())
        );
    }
    Ok(())
}

/// Whether the config file is missing top-level sections that the current
/// schema introduces. Formatting, comments, and value differences are ignored,
/// so a hand-edited config that is still structurally current is not flagged.
fn is_config_up_to_date(source: &str) -> bool {
    if source.trim().is_empty() {
        return true;
    }
    let Ok((upgraded, _)) = upgrade_content(source) else {
        return false;
    };
    let Ok(current) = upgraded.parse::<toml::Table>() else {
        return false;
    };
    let Ok(user) = source.parse::<toml::Table>() else {
        return false;
    };
    current.keys().all(|key| user.contains_key(key))
}

/// Returns `true` when the file's content is not current and the hint should
/// be printed now. Never writes to `path`.
fn check_outdated_with(
    path: &Utf8Path,
    marker_key: &str,
    is_up_to_date: impl Fn(&str) -> bool,
) -> eyre::Result<bool> {
    let Some(current) = std::fs::metadata(path.as_std_path())
        .ok()
        .and_then(|meta| environment::SourceMeta::from_metadata(&meta))
    else {
        return Ok(false);
    };

    let marker_path = hint_marker_path(marker_key);
    if read_marker(marker_path.as_path()) == Some(current) {
        // The file's (mtime, size) is unchanged since we last acted on it, so
        // we already hinted (or verified it) for this exact file state.
        return Ok(false);
    }

    let content = std::fs::read_to_string(path.as_std_path())
        .wrap_err_with(|| format!("failed to read file `{}`", path))?;
    write_marker(marker_path.as_path(), current);
    Ok(!is_up_to_date(&content))
}

fn hint_marker_path(key: &str) -> Utf8PathBuf {
    let path = environment::hash_dir()
        .join("hints")
        .join(format!("{key}.hint"));
    environment::create_parent_dirs(path.as_path());
    path
}

fn read_marker(path: &Utf8Path) -> Option<environment::SourceMeta> {
    let text = std::fs::read_to_string(path).ok()?;
    let (modified_ns, size) = text.trim().split_once(':')?;
    Some(environment::SourceMeta {
        modified_ns: modified_ns.parse().ok()?,
        size: size.parse().ok()?,
    })
}

fn write_marker(path: &Utf8Path, meta: environment::SourceMeta) {
    let content = format!("{}:{}", meta.modified_ns, meta.size);
    if let Err(err) = std::fs::write(path, content) {
        color_print::ceprintln!(
            "<dim>[upgrade] Warning: failed to write hint marker `{}`: {}</>",
            path,
            err
        );
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    fn case_dir(name: &str) -> Utf8PathBuf {
        crate::test_io::case_dir(&format!("upgrade-hint-{name}"))
    }

    #[test]
    fn test_check_outdated_with_up_to_date_content() {
        let root = case_dir("ok");
        fs::create_dir_all(root.as_std_path()).unwrap();

        crate::environment::with_test_environment(
            root.clone(),
            crate::environment::BuildMode::Publish,
            || {
                let path = root.join("file.txt");
                fs::write(path.as_std_path(), "abc").unwrap();

                let outdated =
                    check_outdated_with(path.as_path(), "ok", |content| content == "abc").unwrap();
                assert!(!outdated);
            },
        );

        let _ = fs::remove_dir_all(root.as_std_path());
    }

    #[test]
    fn test_check_outdated_with_hints_once_per_file_state() {
        let root = case_dir("warn-once");
        fs::create_dir_all(root.as_std_path()).unwrap();

        crate::environment::with_test_environment(
            root.clone(),
            crate::environment::BuildMode::Publish,
            || {
                let path = root.join("file.txt");
                fs::write(path.as_std_path(), "xyz").unwrap();

                assert!(
                    check_outdated_with(path.as_path(), "warn", |content| content == "abc")
                        .unwrap()
                );
                // Same (mtime, size): already hinted, do not nag again.
                assert!(
                    !check_outdated_with(path.as_path(), "warn", |content| content == "abc")
                        .unwrap()
                );

                // Touch the file: new file state, hint again.
                fs::write(path.as_std_path(), "xyz").unwrap();
                assert!(
                    check_outdated_with(path.as_path(), "warn", |content| content == "abc")
                        .unwrap()
                );
            },
        );

        let _ = fs::remove_dir_all(root.as_std_path());
    }

    #[test]
    fn test_check_outdated_with_skips_recheck_via_marker() {
        let root = case_dir("marker-skip");
        fs::create_dir_all(root.as_std_path()).unwrap();

        crate::environment::with_test_environment(
            root.clone(),
            crate::environment::BuildMode::Publish,
            || {
                let path = root.join("file.txt");
                fs::write(path.as_std_path(), "abc").unwrap();

                // First check records the (mtime, size) marker.
                assert!(
                    !check_outdated_with(path.as_path(), "skip", |content| content == "abc")
                        .unwrap()
                );
                // The (mtime, size) marker must short-circuit the content
                // check even though the predicate now says it is outdated.
                assert!(!check_outdated_with(path.as_path(), "skip", |_| false).unwrap());
            },
        );

        let _ = fs::remove_dir_all(root.as_std_path());
    }

    #[test]
    fn test_check_outdated_with_missing_file_is_not_outdated() {
        let root = case_dir("missing");
        fs::create_dir_all(root.as_std_path()).unwrap();

        crate::environment::with_test_environment(
            root.clone(),
            crate::environment::BuildMode::Publish,
            || {
                let path = root.join("missing.txt");
                let outdated =
                    check_outdated_with(path.as_path(), "missing", |content| content == "abc")
                        .unwrap();
                assert!(!outdated);
            },
        );

        let _ = fs::remove_dir_all(root.as_std_path());
    }

    #[test]
    fn test_is_config_up_to_date_accepts_current_shape() {
        let current = toml::to_string(&crate::config::Config::default()).unwrap();
        assert!(is_config_up_to_date(&current));
    }

    #[test]
    fn test_is_config_up_to_date_ignores_comments_and_formatting() {
        let current = toml::to_string(&crate::config::Config::default()).unwrap();
        let commented = format!("# my site comment\n{current}");
        assert!(is_config_up_to_date(&commented));
    }

    #[test]
    fn test_is_config_up_to_date_accepts_empty_source() {
        assert!(is_config_up_to_date(""));
        assert!(is_config_up_to_date("\n\n"));
    }

    #[test]
    fn test_is_config_up_to_date_flags_missing_section() {
        let mut table: toml::Table = toml::to_string(&crate::config::Config::default())
            .unwrap()
            .parse()
            .unwrap();
        table.remove("serve");
        let missing_serve = toml::to_string(&table).unwrap();
        assert!(!is_config_up_to_date(&missing_serve));
    }
}
