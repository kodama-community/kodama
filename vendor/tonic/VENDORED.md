# Vendored tonic

- Upstream: https://github.com/kodama-community/tonic
- Source: local checkout `C:/Users/kokic/source/tonic`
- Revision: `1b4d45e7c6eadadcbc096fe2bcc9818bc43d3350`
- Package version: 0.2.0 (including the unreleased HTML backend at this revision)
- Required Rust version: 1.96.1, as declared by upstream

The tracked `Cargo.toml`, `src/` (including upstream unit tests and the KaTeX
license), `README.md`, `CHANGELOG.md`, and `LICENSE` are copied without changes.
Build artifacts, local configuration, and Git metadata are not vendored.
Kodama enables the dependency-free `html` feature; integration belongs in
`src/markdown/math.rs` in the Kodama source tree.

The HTML backend targets KaTeX 0.19.0 `output: "html"` and needs the matching
CSS and fonts. It does not generate MathML.

## Licensing

Tonic is licensed under AGPL-3.0; see [LICENSE](LICENSE). Its HTML backend also
includes KaTeX-derived code and data covered by the MIT notice in
[src/html/LICENSE-KaTeX](src/html/LICENSE-KaTeX). Preserve both notices.

Kodama's own source remains GPL-3.0. GPLv3 section 13 permits combination with
AGPLv3 code, and the combined work must comply with the applicable AGPLv3
requirements, including its network-interaction provisions. Distributions of
the linked Kodama executable must account for tonic's license even when a site's
`pre-render-math` configuration is disabled: this is a runtime configuration,
not a Cargo feature that removes the dependency.
