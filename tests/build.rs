// Copyright (c) 2025 Kodama Project. All rights reserved.
// Released under the GPL-3.0 license as described in the file LICENSE.

mod common;

use common::Site;
use std::{
    fs,
    time::{Duration, SystemTime},
};

impl Site {
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
    site.write("trees/child.md", "# Child\n\nBEFORECHANGE\n");
    site.write("trees/other.md", "# Other\n\nUnrelated\n");
    site.build();
    let pages: Vec<_> = ["index", "child", "other"]
        .into_iter()
        .map(|slug| (slug, site.mark_page(slug)))
        .collect();

    site.write("trees/child.md", "# Child\n\nAFTERCHANGE\n");
    site.build();
    for (slug, modified) in pages {
        let path = site.0.join(format!("publish/{slug}.html"));
        let updated = fs::metadata(&path).unwrap().modified().unwrap();
        if slug == "other" {
            assert_eq!(updated, modified);
        } else {
            assert_ne!(updated, modified);
            let html = fs::read_to_string(path).unwrap();
            assert!(html.contains("AFTERCHANGE"));
            assert!(!html.contains("BEFORECHANGE"));
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
