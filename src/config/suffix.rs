// Copyright (c) 2025 Kodama Project. All rights reserved.
// Released under the GPL-3.0 license as described in the file LICENSE.
// Authors: Kokic (@kokic)

use serde::{Deserialize, Serialize};

use crate::slug::{SectionKind, SourceRole};

pub const DEFAULT_MARKDOWN_SUFFIX: &str = "md";
pub const DEFAULT_TYPST_SUFFIX: &str = "typst";
pub const DEFAULT_TYPST_LIB_SUFFIX: &str = "typ";

/// File suffixes for the different kinds of source documents.
#[derive(Deserialize, Debug, Serialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct Suffix {
    /// Suffix of markdown content documents (sections).
    pub markdown: String,
    /// Suffix of typst content documents (sections).
    pub typst: String,
    /// Suffix of typst library/resource files (compiled to SVG), kept distinct
    /// from the content suffixes so a file's role is unambiguous.
    pub typst_lib: String,
}

impl Default for Suffix {
    fn default() -> Self {
        Self {
            markdown: DEFAULT_MARKDOWN_SUFFIX.to_string(),
            typst: DEFAULT_TYPST_SUFFIX.to_string(),
            typst_lib: DEFAULT_TYPST_LIB_SUFFIX.to_string(),
        }
    }
}

impl Suffix {
    /// The concrete file suffix for a section of the given content kind.
    pub fn suffix_for(&self, kind: SectionKind) -> &str {
        match kind {
            SectionKind::Markdown => &self.markdown,
            SectionKind::Typst => &self.typst,
        }
    }

    /// Classify a file extension into its source role.
    pub fn classify(&self, ext: &str) -> SourceRole {
        if !self.typst_lib.is_empty() && self.typst_lib == ext {
            SourceRole::TypstLib
        } else if self.markdown == ext {
            SourceRole::Markdown
        } else if self.typst == ext {
            SourceRole::Typst
        } else {
            SourceRole::Unknown
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_default_suffixes() {
        let suffix = Suffix::default();
        assert_eq!(suffix.classify("md"), SourceRole::Markdown);
        assert_eq!(suffix.classify("typst"), SourceRole::Typst);
        assert_eq!(suffix.classify("typ"), SourceRole::TypstLib);
        assert_eq!(suffix.classify("unknown"), SourceRole::Unknown);
        assert_eq!(suffix.suffix_for(SectionKind::Markdown), "md");
        assert_eq!(suffix.suffix_for(SectionKind::Typst), "typst");
    }

    #[test]
    fn test_classify_custom_suffixes() {
        let suffix = Suffix {
            markdown: "markdown".to_string(),
            typst: "typ".to_string(),
            typst_lib: "lib".to_string(),
        };
        assert_eq!(suffix.classify("markdown"), SourceRole::Markdown);
        assert_eq!(suffix.classify("typ"), SourceRole::Typst);
        assert_eq!(suffix.classify("lib"), SourceRole::TypstLib);
        assert_eq!(suffix.classify("typst"), SourceRole::Unknown);
        assert_eq!(suffix.classify("md"), SourceRole::Unknown);
        assert_eq!(suffix.suffix_for(SectionKind::Markdown), "markdown");
        assert_eq!(suffix.suffix_for(SectionKind::Typst), "typ");
    }

    #[test]
    fn test_classify_empty_typst_lib_disables_lib_role() {
        let suffix = Suffix {
            typst: "typ".to_string(),
            typst_lib: String::new(),
            ..Suffix::default()
        };
        assert_eq!(suffix.classify("typ"), SourceRole::Typst);
        assert_eq!(suffix.classify("typst"), SourceRole::Unknown);
    }
}
