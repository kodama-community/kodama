# Kodama Flavored CommonMark

This specification amends [CommonMark 0.31.2](https://spec.commonmark.org/0.31.2/). Unless stated below, the CommonMark specification applies.

- Reference-link, reference-image, and footnote labels are case-sensitive, without Unicode case folding.
- Math delimiters no longer have whitespace or word-boundary restrictions.
- Headings use ATX syntax (`#` through `######`) only; Setext headings are not recognized.
- Code blocks use backtick fences only; tilde fences and indented code blocks are not supported.
- Thematic breaks use only `-` markers; `*` and `_` markers are not recognized as thematic breaks.
- Emphasis delimiters no longer have whitespace, punctuation, or word-boundary restrictions. Emphasis cannot nest; mismatched or unclosed delimiters and runs of three or more delimiters are syntax errors.
