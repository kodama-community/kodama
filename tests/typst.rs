// Copyright (c) 2025 Kodama Project. All rights reserved.
// Released under the GPL-3.0 license as described in the file LICENSE.

use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    time::{Duration, SystemTime},
};

struct Site(PathBuf);

impl Site {
    fn new() -> Self {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(".local/test-io")
            .join(format!(
                "typst-{}-{:x}",
                std::process::id(),
                fastrand::u64(..)
            ));
        fs::create_dir_all(root.join("trees/_lib")).unwrap();
        fs::create_dir_all(root.join("assets")).unwrap();
        fs::write(root.join("Kodama.toml"), "").unwrap();
        fs::write(
            root.join("trees/_lib/kodama.typ"),
            include_str!("../src/include/kodama.typ"),
        )
        .unwrap();
        Self(root)
    }

    fn write(&self, path: &str, text: &str) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    fn run(&self, args: &[&str]) -> Output {
        self.command(args).output().unwrap()
    }

    fn command(&self, args: &[&str]) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_kodama"));
        command.args(args).current_dir(&self.0);
        command
    }

    fn build(&self) {
        let output = self.run(&["build"]);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn mark_page(&self, slug: &str) -> SystemTime {
        let path = self.0.join(format!("publish/{slug}.html"));
        fs::OpenOptions::new()
            .write(true)
            .open(&path)
            .unwrap()
            .set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(1_600_000_000))
            .unwrap();
        fs::metadata(path).unwrap().modified().unwrap()
    }
}

#[test]
fn markdown_only_builds_do_not_require_typst() {
    let site = Site::new();
    site.write("trees/index.md", "# Plain Markdown\n\nContent\n");
    for _ in 0..2 {
        let output = site.command(&["build"]).env("PATH", "").output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn unchanged_builds_preserve_page_modification_times() {
    let site = Site::new();
    site.write("trees/index.md", "# Root\n\n[](/child.md#:embed)\n");
    site.write("trees/child.md", "# Child\n\nContent\n");
    site.write("trees/other.md", "# Other\n\nUnrelated\n");
    site.build();

    let pages: Vec<_> = ["index", "child", "other"]
        .into_iter()
        .map(|slug| (slug, site.mark_page(slug)))
        .collect();
    for args in [vec!["build"], vec!["build", "--no-cache"]] {
        let output = site.run(&args);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        for (slug, modified) in &pages {
            let path = site.0.join(format!("publish/{slug}.html"));
            assert_eq!(fs::metadata(path).unwrap().modified().unwrap(), *modified);
        }
    }

    site.write("trees/other.md", "# Other\n\nUnrelated\n\n\n");
    site.build();
    for (slug, modified) in pages {
        let path = site.0.join(format!("publish/{slug}.html"));
        assert_eq!(fs::metadata(path).unwrap().modified().unwrap(), modified);
    }
}

#[test]
fn changed_pages_and_embeds_update_without_rewriting_unrelated_pages() {
    let site = Site::new();
    site.write("trees/index.md", "# Root\n\n[](/child.md#:embed)\n");
    site.write("trees/child.md", "# Child\n\nBEFORE_CHANGE\n");
    site.write("trees/other.md", "# Other\n\nUnrelated\n");
    site.build();
    let pages: Vec<_> = ["index", "child", "other"]
        .into_iter()
        .map(|slug| (slug, site.mark_page(slug)))
        .collect();

    site.write("trees/child.md", "# Child\n\nAFTER_CHANGE\n");
    site.build();
    for (slug, modified) in pages {
        let path = site.0.join(format!("publish/{slug}.html"));
        let updated = fs::metadata(&path).unwrap().modified().unwrap();
        if slug == "other" {
            assert_eq!(updated, modified);
        } else {
            assert_ne!(updated, modified);
            let html = fs::read_to_string(path).unwrap();
            assert!(html.contains("AFTER_CHANGE"));
            assert!(!html.contains("BEFORE_CHANGE"));
        }
    }

    site.write("Kodama.toml", "[kodama]\nbase-url = \"/updated/\"\n");
    site.build();
    for slug in ["index", "child", "other"] {
        let path = site.0.join(format!("publish/{slug}.html"));
        assert!(fs::read_to_string(path)
            .unwrap()
            .contains("href=\"/updated/main.css\""));
    }
}

#[test]
fn missing_or_corrupted_pages_are_rebuilt_from_unchanged_sources() {
    let site = Site::new();
    site.write("trees/index.md", "# Root\n\nOriginal\n");
    site.build();
    let path = site.0.join("publish/index.html");
    let original = fs::read(&path).unwrap();

    fs::remove_file(&path).unwrap();
    site.build();
    assert_eq!(fs::read(&path).unwrap(), original);

    let mut corrupted = original.clone();
    corrupted[0] = b'?';
    fs::write(&path, corrupted).unwrap();
    site.build();
    assert_eq!(fs::read(&path).unwrap(), original);

    fs::write(&path, [0xff, 0xfe]).unwrap();
    site.build();
    assert_eq!(fs::read(&path).unwrap(), original);
}

#[test]
#[ignore = "requires Typst on PATH"]
fn imported_typst_files_invalidate_cached_pages() {
    let site = Site::new();
    site.write("trees/_lib/shared.typ", "#let message = [BEFORE_IMPORT]");
    site.write(
        "trees/index.typst",
        "#import \"_lib/shared.typ\": message\n#message",
    );
    site.build();
    assert!(fs::read_to_string(site.0.join("publish/index.html"))
        .unwrap()
        .contains("BEFORE_IMPORT"));

    site.write(
        "trees/_lib/shared.typ",
        "#let message = [AFTER_IMPORT_CHANGED]",
    );
    site.build();
    let page = fs::read_to_string(site.0.join("publish/index.html")).unwrap();
    assert!(page.contains("AFTER_IMPORT_CHANGED"));
    assert!(!page.contains("BEFORE_IMPORT"));
    fs::remove_file(site.0.join("trees/_lib/shared.typ")).unwrap();
    assert!(!site.run(&["build"]).status.success());
}

#[test]
#[ignore = "requires Typst on PATH"]
fn imported_typst_files_invalidate_svg_artifacts() {
    let site = Site::new();
    site.write("trees/index.md", "# SVG imports");
    site.write("trees/_lib/shared.typ", "#let radius = 10pt");
    site.write(
        "trees/shape.typ",
        "#import \"_lib/shared.typ\": radius\n#circle(radius: radius)",
    );
    site.build();
    let original = fs::read(site.0.join("publish/shape.svg")).unwrap();
    site.write("trees/_lib/shared.typ", "#let radius = 20pt");
    site.build();
    assert_ne!(
        fs::read(site.0.join("publish/shape.svg")).unwrap(),
        original
    );
}

#[test]
#[ignore = "requires Typst on PATH"]
fn imported_typst_files_invalidate_html_fragments() {
    let site = Site::new();
    site.write(
        "trees/index.md",
        "# HTML imports\n\n[fragment](_lib/fragment.typ#:html)",
    );
    site.write(
        "trees/_lib/fragment.typ",
        "#import \"shared.typ\": message\n#message",
    );
    site.write("trees/_lib/shared.typ", "#let message = [BEFORE_FRAGMENT]");
    site.build();
    site.write(
        "trees/_lib/shared.typ",
        "#let message = [AFTER_FRAGMENT_CHANGED]",
    );
    site.build();
    let page = fs::read_to_string(site.0.join("publish/index.html")).unwrap();
    assert!(page.contains("AFTER_FRAGMENT_CHANGED"));
    assert!(!page.contains("BEFORE_FRAGMENT"));
}

#[test]
#[ignore = "requires Typst on PATH"]
fn shared_typst_imports_invalidate_markdown_inline_content() {
    let site = Site::new();
    site.write(
        "trees/index.md",
        "# Inline imports\n\n[message](/_lib/shared.typ#:shared)\n\n[#message](inline)",
    );
    site.write("trees/_lib/shared.typ", "#let message = [BEFORE_INLINE]");
    site.build();
    site.write(
        "trees/_lib/shared.typ",
        "#let message = [AFTER_INLINE_CHANGED]",
    );
    site.build();
    let page = fs::read_to_string(site.0.join("publish/index.html")).unwrap();
    assert!(page.contains("AFTER_INLINE_CHANGED"));
    assert!(!page.contains("BEFORE_INLINE"));
}

#[test]
#[ignore = "requires Typst on PATH"]
fn unrelated_typst_pages_keep_their_cached_entries() {
    let site = Site::new();
    site.write("trees/index.typst", "First page");
    site.write("trees/other.typst", "Unrelated page");
    site.build();
    let path = site.0.join(".cache/entry/other.typst.entry");
    let before = fs::metadata(&path).unwrap().modified().unwrap();
    site.write("trees/index.typst", "First page changed");
    site.build();
    assert_eq!(fs::metadata(path).unwrap().modified().unwrap(), before);
}

#[test]
#[ignore = "requires Typst on PATH"]
fn serve_refreshes_inline_content_and_svgs_after_import_changes() {
    use std::{
        io::{BufRead, BufReader},
        process::{Child, Stdio},
        sync::mpsc,
        thread,
        time::Duration,
    };
    struct Preview(Child, Option<thread::JoinHandle<()>>);
    impl Drop for Preview {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
            if let Some(reader) = self.1.take() {
                let _ = reader.join();
            }
        }
    }
    let site = Site::new();
    site.write(
        "trees/index.md",
        "# Watch imports\n\n[message](/_lib/shared.typ#:shared)\n\n[#message](inline)",
    );
    site.write(
        "trees/shape.typ",
        "#import \"_lib/shared.typ\": radius\n#circle(radius: radius)",
    );
    site.write(
        "trees/_lib/shared.typ",
        "#let message = [BEFORE_WATCH]\n#let radius = 10pt",
    );
    let mut child = site
        .command(&["serve", "--no-server", "--print-json"])
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    let stdout = child.stdout.take().unwrap();
    let (sender, receiver) = mpsc::channel();
    let reader = thread::spawn(move || {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            if sender.send(line).is_err() {
                break;
            }
        }
    });
    let _preview = Preview(child, Some(reader));
    let event = |name| loop {
        let line = receiver
            .recv_timeout(Duration::from_secs(20))
            .expect("preview event timed out");
        if serde_json::from_str::<serde_json::Value>(&line)
            .is_ok_and(|value| value["event"] == name)
        {
            break;
        }
    };
    event("ready");
    let original = fs::read(site.0.join(".cache/publish/shape.svg")).unwrap();
    site.write(
        "trees/_lib/shared.typ",
        "#let message = [AFTER_WATCH_CHANGED]\n#let radius = 20pt",
    );
    event("rebuilt");
    let page = fs::read_to_string(site.0.join(".cache/publish/index.html")).unwrap();
    assert!(page.contains("AFTER_WATCH_CHANGED"));
    assert!(!page.contains("BEFORE_WATCH"));
    assert_ne!(
        fs::read(site.0.join(".cache/publish/shape.svg")).unwrap(),
        original
    );
}

#[cfg(unix)]
#[test]
#[ignore = "requires Typst on PATH"]
fn renderer_identity_changes_invalidate_cached_pages() {
    use std::os::unix::fs::PermissionsExt;
    let site = Site::new();
    site.write("trees/index.typst", "Versioned content");
    let binary = std::env::split_paths(&std::env::var_os("PATH").unwrap())
        .map(|dir| dir.join("typst"))
        .find(|path| path.is_file())
        .unwrap();
    let quote =
        |path: &std::path::Path| format!("'{}'", path.to_str().unwrap().replace('\'', "'\\''"));
    let wrapper = site.0.join("bin/typst");
    let log = site.0.join("compile.log");
    let script = |version| {
        format!("#!/bin/sh\nif [ \"$1\" = --version ]; then\n  printf 'typst {version} cache-test\\n'\nelse\n  printf 'compile\\n' >> {}\n  exec {} \"$@\"\nfi\n", quote(&log), quote(&binary))
    };
    site.write("bin/typst", &script("0.15.0"));
    fs::set_permissions(&wrapper, fs::Permissions::from_mode(0o755)).unwrap();
    let build = || {
        let output = site
            .command(&["build"])
            .env("PATH", site.0.join("bin"))
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    build();
    build();
    assert_eq!(fs::read_to_string(&log).unwrap().lines().count(), 1);
    site.write("bin/typst", &script("0.15.1"));
    build();
    assert_eq!(fs::read_to_string(&log).unwrap().lines().count(), 2);
    // The compiler output is only a cache identity, not a version requirement.
    site.write("bin/typst", &script("local-build"));
    build();
    assert_eq!(fs::read_to_string(&log).unwrap().lines().count(), 3);
}

impl Drop for Site {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
#[ignore = "requires Typst on PATH"]
fn inline_dependency_cache_grows_linearly() {
    fn cache_totals(directory: &std::path::Path) -> (usize, usize) {
        fn records(value: &serde_json::Value) -> usize {
            match value {
                serde_json::Value::Object(object) => {
                    object
                        .get("files")
                        .and_then(serde_json::Value::as_object)
                        .map_or(0, |files| files.len())
                        + object.values().map(records).sum::<usize>()
                }
                serde_json::Value::Array(array) => array.iter().map(records).sum(),
                _ => 0,
            }
        }
        let (mut count, mut bytes) = (0, 0);
        for entry in fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                let (nested_count, nested_bytes) = cache_totals(&path);
                count += nested_count;
                bytes += nested_bytes;
            } else {
                let data = fs::read(&path).unwrap();
                bytes += data.len();
                if let Ok(value) = serde_json::from_slice(&data) {
                    count += records(&value);
                }
            }
        }
        (count, bytes)
    }

    let mut sizes = Vec::new();
    for count in [32, 64] {
        let site = Site::new();
        for index in 0..count {
            let slug = if index == 0 {
                "index".to_string()
            } else {
                format!("p{index}")
            };
            site.write(
                &format!("trees/_lib/value{index}.typ"),
                &format!("#let message = [Value {index}]"),
            );
            site.write(&format!("trees/{slug}.md"), &format!("# Page {index}\n\n[message](/_lib/value{index}.typ#:shared)\n\n[#message](inline)"));
        }
        site.build();
        let (records, bytes) = cache_totals(&site.0.join(".cache"));
        assert_eq!(records, count, "each imported file should be recorded once");
        sizes.push(bytes);
        site.build();
        assert_eq!(cache_totals(&site.0.join(".cache")).0, count);
    }
    assert!(
        sizes[1] < sizes[0] * 3,
        "doubling pages must not quadruple the cache: {sizes:?}"
    );
}

#[test]
#[ignore = "requires Typst on PATH"]
fn inline_dependencies_survive_partial_rebuilds() {
    let site = Site::new();
    let first = "# First\n\n[message](/_lib/first.typ#:shared)\n\n[#message](inline)";
    site.write("trees/index.md", first);
    site.write(
        "trees/other.md",
        "# Other\n\n[message](/_lib/other.typ#:shared)\n\n[#message](inline)",
    );
    site.write("trees/_lib/first.typ", "#let message = [FIRST]");
    site.write("trees/_lib/other.typ", "#let message = [BEFORE_OTHER]");
    site.build();
    let other_cache = site.0.join(".cache/entry/other.md.entry");
    let original = fs::metadata(&other_cache).unwrap().modified().unwrap();

    site.write("trees/index.md", &format!("{first}\n\nEdited first page."));
    site.build();
    site.build();
    assert_eq!(
        fs::metadata(&other_cache).unwrap().modified().unwrap(),
        original
    );
    site.write(
        "trees/_lib/other.typ",
        "#let message = [AFTER_OTHER_CHANGED]",
    );
    site.build();
    assert!(fs::read_to_string(site.0.join("publish/other.html"))
        .unwrap()
        .contains("AFTER_OTHER_CHANGED"));
    assert!(fs::read_to_string(site.0.join("publish/index.html"))
        .unwrap()
        .contains("Edited first page."));

    fs::remove_file(site.0.join(".cache/hash/typst/inline.dependencies")).unwrap();
    site.build();
    assert!(fs::read_to_string(site.0.join("publish/other.html"))
        .unwrap()
        .contains("AFTER_OTHER_CHANGED"));
}

#[test]
#[ignore = "requires Typst on PATH"]
fn stale_inline_entries_do_not_reuse_a_new_dependency_report() {
    let site = Site::new();
    site.write(
        "trees/index.md",
        "# Inline\n\n[message](/_lib/shared.typ#:shared)\n\n[#message](inline)",
    );
    site.write("trees/_lib/shared.typ", "#let message = [BEFORE_REPORT]");
    site.build();
    let path = site.0.join(".cache/entry/index.md.entry");
    let stale = fs::read(&path).unwrap();
    site.write(
        "trees/_lib/shared.typ",
        "#let message = [AFTER_REPORT_CHANGED]",
    );
    site.build();
    // An interrupted build can leave old page entries beside a newer shared report.
    fs::write(path, stale).unwrap();
    site.build();
    let page = fs::read_to_string(site.0.join("publish/index.html")).unwrap();
    assert!(page.contains("AFTER_REPORT_CHANGED"));
    assert!(!page.contains("BEFORE_REPORT"));
}

#[test]
#[ignore = "requires Typst on PATH"]
fn inline_check_does_not_write_shared_cache() {
    let site = Site::new();
    site.write(
        "trees/index.md",
        "# Check\n\n[message](/_lib/shared.typ#:shared)\n\n[#message](inline)",
    );
    site.write("trees/_lib/shared.typ", "#let message = [CHECK_ONLY]");
    let output = site.run(&["check", "--strict"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!site.0.join(".cache").exists());
    assert!(!site.0.join("publish").exists());
}

#[test]
#[ignore = "requires Typst on PATH"]
fn svg_errors_fail_build_and_are_retried() {
    let site = Site::new();
    site.write("trees/index.md", "# SVG\n\n![shape](shape.svg)\n");
    site.write("trees/shape.typ", "#circle(radius: 10pt)");
    site.build();
    let original = fs::read(site.0.join("publish/shape.svg")).unwrap();

    site.write("trees/shape.typ", "#unknown_shape()");
    for _ in 0..2 {
        assert!(!site.run(&["build"]).status.success());
    }
    assert_eq!(
        fs::read(site.0.join("publish/shape.svg")).unwrap(),
        original
    );

    site.write("trees/shape.typ", "#circle(radius: 20pt)");
    site.build();
    assert_ne!(
        fs::read(site.0.join("publish/shape.svg")).unwrap(),
        original
    );
}

#[test]
#[ignore = "requires Typst on PATH"]
fn inline_errors_do_not_become_successful_cached_pages() {
    let site = Site::new();
    site.write("trees/index.md", "# Inline\n\n[#unknown_formula](inline)\n");
    for _ in 0..2 {
        assert!(!site.run(&["build"]).status.success());
        assert!(!site.0.join(".cache/entry/index.md.entry").exists());
    }
    site.write("trees/index.md", "# Inline\n\n[x+y](inline-math)\n");
    site.build();
}

#[test]
#[ignore = "requires Typst on PATH"]
fn html_errors_fail_build_and_are_retried() {
    let site = Site::new();
    site.write(
        "trees/index.md",
        "# HTML\n\n[fragment](_lib/fragment.typ#:html)\n",
    );
    site.write("trees/_lib/fragment.typ", "Original fragment");
    site.build();

    site.write("trees/_lib/fragment.typ", "#unknown_fragment()");
    // Reparse the Markdown source so it asks for the changed HTML fragment.
    site.write(
        "trees/index.md",
        "# Changed HTML\n\n[fragment](_lib/fragment.typ#:html)\n",
    );
    for _ in 0..2 {
        assert!(!site.run(&["build"]).status.success());
    }
}

#[test]
#[ignore = "requires Typst on PATH"]
fn typst_pages_and_embeds_have_one_document_shell() {
    let site = Site::new();
    let header = "#import \"_lib/kodama.typ\": *\n#show: kodama\n";
    site.write(
        "trees/index.typst",
        &format!("{header}#metadata((title: \"Root\\\\\",))\n$x+y$\n#embed(\"child\", \"Child\")"),
    );
    site.write(
        "trees/child.typst",
        &format!("{header}#metadata((title: \"Child\",))\n$a+b$"),
    );
    for _ in 0..2 {
        site.build();
        let page = fs::read_to_string(site.0.join("publish/index.html")).unwrap();
        assert_eq!(page.matches("<head>").count(), 1);
        assert_eq!(page.matches("<body>").count(), 1);
        assert!(!page.contains("data-kodama-typst"));
        assert!(page.contains("<svg"));
    }
}
