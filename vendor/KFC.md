# Kodama Flavored CommonMark

This specification amends [CommonMark 0.31.2](https://spec.commonmark.org/0.31.2/). Unless stated below, the CommonMark specification applies.

- Reference-link, reference-image, and footnote labels are case-sensitive, without Unicode case folding.
- Named and numeric HTML character references are not decoded in Markdown, including in link destinations and titles. `&copy;`, `&#169;`, and `&#xA9;` remain as written; use `©` for the character itself. Write Markdown URLs as `/?a=1&b=2`, not `/?a=1&amp;b=2`. This does not affect raw HTML or the escaping of text and attribute values when rendering to HTML.
- Math delimiters no longer have whitespace or word-boundary restrictions.
- Markdown delimiters do not produce superscripts or subscripts. Write them in math spans, for example `$x_i^2$`, `$\mathrm{H}^{+}$`, or `$\mathrm{NH}_4^{+}$`. KFC identifies the math span; the TeX renderer interprets `^` and `_` inside it. Outside math, `^` is not a formatting delimiter and `_…_` remains emphasis.
- When strikethrough is enabled, `~` and `~~` are both strikethrough delimiters with no whitespace, punctuation, or word-boundary restrictions. Delimiters pair only with runs of the same length and can nest. Unmatched or escaped delimiters, and tildes inside code or math, are not parsed as strikethrough. The emphasis-specific syntax errors do not apply to these delimiters.
- Headings use ATX syntax (`#` through `######`) only; Setext headings are not recognized.
- Code blocks use backtick fences only; tilde fences and indented code blocks are not supported.
- Thematic breaks use only `-` markers; `*` and `_` markers are not recognized as thematic breaks.
- Emphasis delimiters no longer have whitespace, punctuation, or word-boundary restrictions. Emphasis cannot nest; mismatched or unclosed delimiters and runs of three or more delimiters are syntax errors.
- `==…==` is ordinary text, not an inline highlight delimiter.
