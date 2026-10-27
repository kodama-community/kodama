// Copyright (c) 2025 Kodama Project. All rights reserved.
// Released under the GPL-3.0 license as described in the file LICENSE.

mod common;

use std::{
    fs,
    io::{BufRead, BufReader},
    process::Stdio,
};

use common::Site;

#[test]
fn no_config_serve_reports_null_config_and_watches_no_config_file() {
    let site = Site::new();
    fs::remove_file(site.0.join("Kodama.toml")).unwrap();
    site.write("trees/index.md", "# Root\n\nContent\n");

    let mut child = site
        .command(&[
            "serve",
            "--no-config",
            "--no-server",
            "--disable-reload",
            "--print-json",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let mut line = String::new();
    BufReader::new(child.stdout.take().unwrap())
        .read_line(&mut line)
        .unwrap();
    let _ = child.kill();
    let output = child.wait_with_output().unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(line.contains("\"event\":\"ready\""), "{line}\n{stderr}");
    assert!(line.contains("\"config\":null"), "{line}");
    // Without a config file there is no config path to watch, so none may be
    // mentioned while setting up the watch.
    assert!(!stderr.contains("Kodama.toml"), "{stderr}");
}
