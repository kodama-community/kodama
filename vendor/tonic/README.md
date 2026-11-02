# tonic

A Rust port of the [KaTeX](https://katex.org) parser: lexes and parses LaTeX
math expressions with full macro expansion into a typed [`ParseNode`] AST, and
renders that AST to Unicode text.

See the [changelog](../CHANGELOG.md) for release notes and migration guidance.

## Usage

```rust
use katex_parser::{parse, render, RenderConfig, Settings};

// Parse LaTeX into a typed AST.
let nodes = parse(r"\frac{1}{2} + \sqrt[3]{x}", &mut Settings::new())?;

// Render the AST as Unicode text.
let text = render(&nodes, RenderConfig::new());
assert_eq!(text, "1∕2 + ³√x");

# Ok::<(), katex_parser::ParseError>(())
```

`Settings` configures display mode, user macros, strictness, trust policy, and a
persistent macro store shared across parses:

```rust
use katex_parser::{parse, render, Macros, Settings};
use std::collections::HashMap;

let mut settings = Settings::new();
settings.display_mode = true; // display-style fractions/matrices
settings.macro_store = Some(Macros::new(HashMap::new()));
settings.global_group = true; // persist \newcommand across parses

parse(r"\newcommand{\R}{\mathbb{R}}", &mut settings)?;
let nodes = parse(r"\R", &mut settings)?; // \R is still defined
```

## Features

- **Typed AST** — every construct becomes a [`ParseNode`] variant
  (`GenFrac`, `Sqrt`, `Array`, `Op`, `SupSub`, …), mirroring KaTeX.
- **Macro expansion** — `\def`/`\gdef`/`\edef`/`\xdef`, `\newcommand`,
  argument substitution, delimiter arguments, `\@ifnextchar`, `\noexpand`.
- **Environments** — `array`, `matrix`, `cases`, `aligned`/`align`, `gather`,
  `equation`, `subarray`, and commutative-diagram `CD`.
- **Unicode rendering backend** — block fractions, matrices, boxes, and
  commutative diagrams with baseline alignment, plus font mapping
  (`\mathbb`, `\mathbf`, `\mathcal`, …) and Unicode script sub/superscripts.
- **Zero dependencies**.

## License

AGPL-3.0. See [LICENSE](LICENSE).

[`ParseNode`]: https://docs.rs/tonic/latest/katex_parser/enum.ParseNode.html
