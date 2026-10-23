use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
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
        Command::new(env!("CARGO_BIN_EXE_kodama"))
            .args(args)
            .current_dir(&self.0)
            .output()
            .unwrap()
    }

    fn build(&self) {
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

#[test]
#[ignore = "requires Typst 0.15+ on PATH"]
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
#[ignore = "requires Typst 0.15+ on PATH"]
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
#[ignore = "requires Typst 0.15+ on PATH"]
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
