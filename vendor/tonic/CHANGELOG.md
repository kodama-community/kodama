# Changelog

## Unreleased

### Added

- Optional, dependency-free `html` feature with `render_html` /
  `HtmlRenderConfig` (also `html::render` / `html::RenderConfig`). The backend
  follows dulox's HTML layout and targets KaTeX 0.19.0 `output: "html"`, including
  fonts, SVG delimiters, accents, arrays, tags, trusted HTML, and layout options.

### Fixed

- Preserve HTML attribute insertion order and accept `{,}` in `\htmlData`.
- Preserve rejected-command metadata for KaTeX-compatible HTML while retaining
  the Unicode backend's existing text fallbacks.
- Include KaTeX's text wrapper around unsupported commands when
  `throw_on_error` is false.

### API changes

- `ParseNode::Html.attributes` is now `Vec<(String, String)>` instead of a
  `HashMap`, because serialization order is part of the HTML contract. The
  `TrustContext::HtmlData` callback still receives a `HashMap`.
- Exhaustive `ParseNode` matches must handle the new `Unsupported` variant.
  HTML uses its `command` and `color`; text backends should use `fallback`.
- `FunctionParser::unsupported_command` has a default implementation. Custom
  parsers can override it to supply their configured error color.

## Rust 0.2.0 - 2026-10-02

### Added

- Parse `\reflectbox` with a text-box argument and `\mathreflectbox` with a math
  argument, producing a `ReflectBox` AST node. The Unicode backends render the
  body without mirroring because plain text cannot mirror arbitrary glyphs.
  Both commands also appear in the MoonBit builtin completion catalogue.
  ([#2](https://github.com/kodama-community/tonic/issues/2))

### Fixed

- Expand the current scoped `\arraystretch` macro and match KaTeX's numeric-prefix
  parsing, including signed infinity and exponent overflow/underflow. Explicit
  environment values such as `cases` and `smallmatrix` retain their fixed stretch.
  ([#1](https://github.com/kodama-community/tonic/issues/1))
- Preserve the distinction between default column spacing and explicitly zero
  spacing. ([#3](https://github.com/kodama-community/tonic/issues/3))
- Match KaTeX's trailing empty rows and per-row manual/automatic tags, including
  `\nonumber` and `\notag`. Keep CD's terminal structural empty row in the AST and
  omit it from Unicode text diagrams.
  ([#4](https://github.com/kodama-community/tonic/issues/4))

### Breaking changes

Both implementations change the following public APIs:

| Field | MoonBit | Rust |
| --- | --- | --- |
| `ArrayColumn::AlignColumn` fields `pre_gap`, `post_gap` | `Double` → `Double?` | `f64` → `Option<f64>` |
| `ArrayEnvironmentOptions.array_stretch` | `Double` → `Double?` | `f64` → `Option<f64>` |
| New `ArrayEnvironmentOptions.empty_single_row` | `Bool` | `bool` |

- For column gaps, `None` means the renderer chooses a default; `Some(0.0)`
  explicitly disables the gap. MoonBit JSON consumers must preserve this
  distinction too.
- For environment stretch, `None` expands the current macro (default `1.0`),
  while `Some(value)` specifies the environment's stretch. The resolved
  `ParseNode::Array.array_stretch` remains `Double` / `f64`.
- Set `empty_single_row` to `true` for AMS-style environments that retain a
  single empty row, and `false` for ordinary arrays.
- Exhaustive `ParseNode` consumers must handle `ReflectBox`:
  `ReflectBox(mode~, body~)` in MoonBit, or `ReflectBox { mode, body }` in Rust.
  `\reflectbox` is also allowed in text; `\mathreflectbox` is only allowed in math.
- Array body length may differ from row-gap, horizontal-line, and tag metadata
  lengths after removal of a trailing empty row. Consumers must preserve this
  metadata and account for CD's structural terminal row.
