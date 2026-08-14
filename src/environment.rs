// Copyright (c) 2025 Kodama Project. All rights reserved.
// Released under the GPL-3.0 license as described in the file LICENSE.
// Authors: Kokic (@kokic), Spore (@s-cerevisiae)

use std::sync::{Arc, LazyLock, OnceLock, RwLock};

use camino::{Utf8Path, Utf8PathBuf};
use eyre::eyre;

use crate::{
    config::{self, Config},
    path_utils,
};

mod cache;
mod config_access;
mod hashing;
mod imports;
mod paths;

pub use cache::ensure_cache_version;
pub use config_access::{
    allow_unsafe_html, asref, assets_dir, assets_dir_without_root, base_url, base_url_raw,
    classify_source, deploy_edit_url, editor_url, elaborate_cjk_text, feed_path, footer_mode,
    footer_sort_by, get_cache_dir, get_edit_text, get_footer_backlinks_text,
    get_footer_references_text, get_toc_text, graph_path, indexes_path, inline_css, inline_script,
    is_short_slug, is_toc_left, is_toc_mobile_sticky, is_toc_sticky, kind_from_extension,
    markdown_suffix, output_dir, pretty_urls, publish_rss, reload_marker_path, serve_command,
    suffix_for_kind, theme_lock, theme_paths, toc_max_width, trees_dir, trees_dir_without_root,
    typst_root_dir, typst_suffix,
};
pub use hashing::{file_meta_updated, relative_source_meta, SourceMeta};
pub use imports::{import_fonts_html, import_math_html, import_meta_html, import_style_html};
pub use paths::{
    create_parent_dirs, entry_dir, entry_file_path, full_html_url, full_url, hash_dir,
    hash_file_path, input_path, output_path,
};

pub struct Environment {
    /// Specifies the project root path.
    ///
    /// Please note that this value should always be automatically derived from
    /// the location of the toml configuration file.
    pub root: Utf8PathBuf,
    pub config_file: Utf8PathBuf,
    pub config: Config,
    pub build_mode: BuildMode,
}

static ENVIRONMENT: OnceLock<RwLock<Environment>> = OnceLock::new();

fn default_environment() -> Environment {
    Environment {
        root: "./".into(),
        config_file: crate::config::DEFAULT_CONFIG_PATH.into(),
        config: Config::default(),
        build_mode: BuildMode::Publish,
    }
}

fn read_environment<R>(lock: &RwLock<Environment>, f: impl FnOnce(&Environment) -> R) -> R {
    match lock.read() {
        Ok(env) => f(&env),
        Err(poisoned) => {
            color_print::ceprintln!(
                "<y>Warning: environment read lock is poisoned; continuing with recovered state.</>"
            );
            let env = poisoned.into_inner();
            f(&env)
        }
    }
}

fn write_environment(lock: &RwLock<Environment>, environment: Environment) {
    match lock.write() {
        Ok(mut env) => {
            *env = environment;
        }
        Err(poisoned) => {
            color_print::ceprintln!(
                "<y>Warning: environment write lock is poisoned; replacing with recovered state.</>"
            );
            let mut env = poisoned.into_inner();
            *env = environment;
        }
    }
}

fn environment_lock(warn_if_uninitialized: bool) -> &'static RwLock<Environment> {
    if warn_if_uninitialized && ENVIRONMENT.get().is_none() {
        color_print::ceprintln!(
            "<y>Warning: environment accessed before initialization; using default configuration.</>"
        );
    }
    ENVIRONMENT.get_or_init(|| RwLock::new(default_environment()))
}

fn update_environment(environment: Environment) {
    let derived = compute_derived(&environment);
    let lock = environment_lock(false);
    write_environment(lock, environment);
    set_derived(derived);
}

/// Derived, hot-path values cached from the current [`Environment`]. The
/// environment only changes via [`update_environment`] (startup, config reload,
/// tests), so these are repopulated there and served to every hot path
/// (URL/link/output path resolution) as an `Arc` snapshot instead of taking the
/// environment read lock and re-deriving per call.
#[derive(Debug)]
struct DerivedCache {
    root_dir: Utf8PathBuf,
    trees_dir: Utf8PathBuf,
    trees_dir_without_root: String,
    assets_dir: Utf8PathBuf,
    assets_dir_without_root: String,
    cache_dir: Utf8PathBuf,
    output_dir: Utf8PathBuf,
    base_url: String,
    pretty_urls: bool,
    short_slug: bool,
}

fn compute_derived(env: &Environment) -> DerivedCache {
    let root = env.root.clone();
    let output = match env.build_mode {
        BuildMode::Publish | BuildMode::Check => env.config.build.output.clone(),
        BuildMode::Serve => env.config.serve.output.clone(),
    };
    let base_url = match env.build_mode {
        BuildMode::Publish | BuildMode::Check => env.config.kodama.base_url.clone(),
        BuildMode::Serve => crate::config::kodama::DEFAULT_BASE_URL.to_string(),
    };
    DerivedCache {
        root_dir: root.clone(),
        trees_dir: root.join(&env.config.kodama.trees),
        trees_dir_without_root: env.config.kodama.trees.clone(),
        assets_dir: root.join(&env.config.kodama.assets),
        assets_dir_without_root: env.config.kodama.assets.clone(),
        cache_dir: root.join(CACHE_DIR_NAME),
        output_dir: root.join(output),
        base_url,
        pretty_urls: env.config.build.pretty_urls,
        short_slug: env.config.build.short_slug,
    }
}

static DERIVED_CACHE: LazyLock<RwLock<Arc<DerivedCache>>> =
    LazyLock::new(|| RwLock::new(Arc::new(compute_derived(&default_environment()))));

/// Cheap, owned snapshot of the cached derived values. The lock is released
/// before the caller clones the fields, so hot paths never hold it.
fn derived_snapshot() -> Arc<DerivedCache> {
    match DERIVED_CACHE.read() {
        Ok(cache) => Arc::clone(&cache),
        Err(poisoned) => Arc::clone(&poisoned.into_inner()),
    }
}

fn set_derived(derived: DerivedCache) {
    let mut cache = match DERIVED_CACHE.write() {
        Ok(cache) => cache,
        Err(poisoned) => poisoned.into_inner(),
    };
    *cache = Arc::new(derived);
}

fn with_environment<R>(f: impl FnOnce(&Environment) -> R) -> R {
    let lock = environment_lock(true);
    read_environment(lock, f)
}

fn with_config<R>(f: impl FnOnce(&Config) -> R) -> R {
    with_environment(|env| f(&env.config))
}

#[cfg(test)]
pub(super) fn with_test_environment<R>(
    root: Utf8PathBuf,
    build_mode: BuildMode,
    f: impl FnOnce() -> R,
) -> R {
    with_test_environment_config(root, build_mode, Config::default(), f)
}

#[cfg(test)]
pub(super) fn with_test_environment_config<R>(
    root: Utf8PathBuf,
    build_mode: BuildMode,
    config: Config,
    f: impl FnOnce() -> R,
) -> R {
    let _guard = lock_test_env_mutex();

    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            update_environment(default_environment());
        }
    }

    let _reset = Reset;
    update_environment(Environment {
        root: root.clone(),
        config_file: root.join(crate::config::DEFAULT_CONFIG_PATH),
        config,
        build_mode,
    });
    f()
}

pub fn init_environment(toml_file: &Utf8Path, build_mode: BuildMode) -> eyre::Result<()> {
    let toml_file = config::find_config(toml_file)?;

    let (root, _file_name) = path_utils::split_file_name(&toml_file)
        .ok_or_else(|| eyre!("invalid config path `{}`: path cannot be empty", toml_file))?;
    let toml = std::fs::read_to_string(&toml_file)?;

    update_environment(Environment {
        root: root.to_owned(),
        config_file: toml_file,
        config: config::parse_config(&toml)?,
        build_mode,
    });
    Ok(())
}

#[cfg(test)]
fn test_env_mutex() -> &'static std::sync::Mutex<()> {
    static TEST_ENV_MUTEX: OnceLock<std::sync::Mutex<()>> = OnceLock::new();
    TEST_ENV_MUTEX.get_or_init(|| std::sync::Mutex::new(()))
}

#[derive(Clone, Copy)]
pub enum BuildMode {
    /// Publish mode for the `kodama build` command.
    Publish,

    /// Check mode for the `kodama check` command.
    Check,

    /// Serve mode for the `kodama serve` command.
    Serve,
}

pub const CACHE_DIR_NAME: &str = ".cache";
pub const HASH_DIR_NAME: &str = "hash";
pub const ENTRY_DIR_NAME: &str = "entry";

pub fn to_page_suffix(pretty_urls: bool) -> String {
    if pretty_urls { "" } else { ".html" }.into()
}

pub fn root_dir() -> Utf8PathBuf {
    derived_snapshot().root_dir.clone()
}

pub fn config_file() -> Utf8PathBuf {
    with_environment(|env| env.config_file.clone())
}

pub fn is_serve() -> bool {
    with_environment(|env| matches!(env.build_mode, BuildMode::Serve))
}

#[allow(dead_code)]
pub fn is_publish() -> bool {
    with_environment(|env| matches!(env.build_mode, BuildMode::Publish))
}

pub fn is_check() -> bool {
    with_environment(|env| matches!(env.build_mode, BuildMode::Check))
}

#[cfg(test)]
fn lock_test_env_mutex() -> std::sync::MutexGuard<'static, ()> {
    match test_env_mutex().lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            color_print::ceprintln!(
                "<y>Warning: test environment mutex is poisoned; continuing with recovered state.</>"
            );
            poisoned.into_inner()
        }
    }
}

/// Mock environment for testing purposes.
#[allow(dead_code)]
pub fn mock_environment() -> eyre::Result<()> {
    #[cfg(test)]
    let _guard = lock_test_env_mutex();

    update_environment(default_environment());
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, RwLock};

    use super::*;

    fn poison_read_lock(lock: Arc<RwLock<Environment>>) {
        let _ = std::thread::spawn(move || {
            let _guard = lock.write().unwrap();
            panic!("poison lock");
        })
        .join();
    }

    #[test]
    fn test_read_environment_recovers_from_poisoned_lock() {
        let lock = Arc::new(RwLock::new(default_environment()));
        poison_read_lock(lock.clone());

        let root = read_environment(&lock, |env| env.root.clone());
        assert_eq!(root, Utf8PathBuf::from("./"));
    }

    #[test]
    fn test_write_environment_recovers_from_poisoned_lock() {
        let lock = Arc::new(RwLock::new(default_environment()));
        poison_read_lock(lock.clone());

        write_environment(
            &lock,
            Environment {
                root: Utf8PathBuf::from("site"),
                config_file: Utf8PathBuf::from("Kodama.toml"),
                config: Config::default(),
                build_mode: BuildMode::Serve,
            },
        );

        let (root, mode) = read_environment(&lock, |env| (env.root.clone(), env.build_mode));
        assert_eq!(root, Utf8PathBuf::from("site"));
        assert!(matches!(mode, BuildMode::Serve));
    }

    #[test]
    fn test_derived_cache_refreshes_on_environment_update() {
        let first = crate::test_io::case_dir("env-derived-first");
        super::with_test_environment(first.clone(), BuildMode::Publish, || {
            assert_eq!(root_dir(), first);
            assert_eq!(
                trees_dir(),
                first.join(crate::config::kodama::DEFAULT_SOURCE_DIR)
            );
            assert_eq!(get_cache_dir(), first.join(CACHE_DIR_NAME));
            assert_eq!(
                base_url(),
                crate::config::kodama::DEFAULT_BASE_URL.to_string()
            );
        });

        let second = crate::test_io::case_dir("env-derived-second");
        super::with_test_environment(second.clone(), BuildMode::Publish, || {
            assert_eq!(root_dir(), second);
            assert_eq!(
                trees_dir(),
                second.join(crate::config::kodama::DEFAULT_SOURCE_DIR)
            );
            assert_eq!(get_cache_dir(), second.join(CACHE_DIR_NAME));
        });

        let _ = std::fs::remove_dir_all(first.as_std_path());
        let _ = std::fs::remove_dir_all(second.as_std_path());
    }

    #[test]
    fn test_derived_cache_reflects_serve_build_mode() {
        let root = crate::test_io::case_dir("env-derived-serve");
        super::with_test_environment(root.clone(), BuildMode::Serve, || {
            assert_eq!(
                base_url(),
                crate::config::kodama::DEFAULT_BASE_URL.to_string()
            );
            assert_eq!(output_dir(), root.join("./.cache/publish"));
        });

        let _ = std::fs::remove_dir_all(root.as_std_path());
    }
}
