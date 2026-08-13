
# Kodama

<img src="https://kodama-community.github.io/assets/kodama.svg" title="kodama" width=150 /> 

A [Typst](https://github.com/typst/typst)-friendly static Zettelkästen site generator.

[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/kokic/kodama)

## Features

- **Small, self-contained binary.** A single command-line executable of a few megabytes, with a deliberately minimal dependency footprint — optional features are disabled across all dependencies, so there is nothing extra to install or carry around.

- **Fast incremental rebuilds.** In `serve` mode, changes are picked up from a watcher-driven dirty set and compiled incrementally against an in-memory session; output pages that are unchanged are skipped via content hashing, so rebuilds touch only what actually changed and avoid redundant file writes.

- **Typst support.** Markdown and Typst sources are compiled using the Typst installation on the user's device and embedded as SVG / HTML, so the full Typst feature set is available. Inline formulas written in Typst receive additional style optimizations.

- **Automatic light and dark themes.** Light/dark theme switching works out of the box, including for formulas and color images produced by Typst. Any detail of the site style can still be adjusted manually, without rebuilding Kodama itself.

- **Forester-style organization.** Markdown files are organized in the manner of [Jon Sterling](https://www.jonmsterling.com/index/index.xml)'s [Forester](https://www.forester-notes.org/index/index.xml).

# Docs

- [Demo](https://kokic.github.io/riemann-surfaces/)
- [Getting Started](/docs/users/getting-started.md)
- [User Documentation](/docs/users/README.md)
- [Developer Documentation](/docs/developers/README.md)

# Community

- QQ Group 1044305570 - 中文交流群. 
- [Kodama Discord server](https://discord.gg/mbeF8J6rXX) - a place to chat with the project's devs and other community members.

