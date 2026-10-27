// Copyright (c) 2025 Kodama Project. All rights reserved.
// Released under the GPL-3.0 license as described in the file LICENSE.

// `Site` is shared by several integration test binaries, so not every helper is
// used by each of them.
#![allow(dead_code)]

use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
};

pub(crate) struct Site(pub(crate) PathBuf);

impl Site {
    pub(crate) fn new() -> Self {
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
            include_str!("../../src/include/kodama.typ"),
        )
        .unwrap();
        Self(root)
    }

    pub(crate) fn write(&self, path: &str, text: &str) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    pub(crate) fn run(&self, args: &[&str]) -> Output {
        self.command(args).output().unwrap()
    }

    pub(crate) fn command(&self, args: &[&str]) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_kodama"));
        command.args(args).current_dir(&self.0);
        command
    }

    pub(crate) fn build(&self) {
        let output = self.run(&["build"]);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

impl Drop for Site {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
