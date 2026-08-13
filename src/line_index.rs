// Copyright (c) 2025 Kodama Project. All rights reserved.
// Released under the GPL-3.0 license as described in the file LICENSE.
// Authors: Kokic (@kokic)

/// Precomputed byte offsets of line starts, used to resolve a byte offset into
/// a 1-based `(line, column)` position.
pub struct LineIndex {
    line_starts: Vec<usize>,
}

impl LineIndex {
    pub fn new(source: &str) -> Self {
        let mut line_starts = vec![0];
        for (offset, byte) in source.bytes().enumerate() {
            if byte == b'\n' {
                line_starts.push(offset + 1);
            }
        }
        Self { line_starts }
    }

    /// Return the 1-based `(line, col)` of the char at byte offset `idx`.
    pub fn line_col_at(&self, source: &str, idx: usize) -> (usize, usize) {
        let idx = idx.min(source.len());
        let line = self.line_starts.partition_point(|&start| start <= idx) - 1;
        let line_start = self.line_starts[line];
        let col = source[line_start..idx].chars().count() + 1;
        (line + 1, col)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_col_at_tracks_line_and_col() {
        let source = "line1\nline2\n中文line3\n";
        let index = LineIndex::new(source);

        assert_eq!(index.line_col_at(source, 0), (1, 1));
        assert_eq!(index.line_col_at(source, 5), (1, 6));
        assert_eq!(index.line_col_at(source, 6), (2, 1));
        assert_eq!(index.line_col_at(source, 11), (2, 6));
        assert_eq!(index.line_col_at(source, 12), (3, 1));

        // Multi-byte chars advance the column by one char each.
        let cjk = "中文";
        assert_eq!(index.line_col_at(source, 12 + cjk.len()), (3, 3));
        assert_eq!(index.line_col_at(source, source.len()), (4, 1));
    }

    #[test]
    fn test_line_col_at_handles_clamped_index() {
        let source = "abc\ndef\n";
        let index = LineIndex::new(source);

        assert_eq!(index.line_col_at(source, 3), (1, 4));
        assert_eq!(index.line_col_at(source, 4), (2, 1));
        assert_eq!(index.line_col_at(source, 7), (2, 4));
        assert_eq!(index.line_col_at(source, 999), (3, 1));
    }
}
