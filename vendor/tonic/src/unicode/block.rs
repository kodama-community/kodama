use crate::unicode::unicode_width::{is_east_asian_wide, is_zero_width_mark};

/// A 2D text block where every line has equal width.
/// Supports vertical stacking, horizontal juxtaposition with baseline
/// alignment, and delimiter wrapping — building blocks for block-mode
/// Unicode rendering of matrices, fractions, delimited expressions, etc.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub lines: Vec<String>,
    pub width: usize,
    pub baseline: usize,
}

impl Block {
    pub fn from(s: &str) -> Block {
        let raw: Vec<&str> = s.split('\n').collect();
        let width = raw.iter().map(|l| display_width(l)).max().unwrap_or(0);
        let lines: Vec<String> = raw.iter().map(|l| pad_right(l, width)).collect();
        Block {
            lines,
            width,
            baseline: 0,
        }
    }

    pub fn empty() -> Block {
        Block {
            lines: Vec::new(),
            width: 0,
            baseline: 0,
        }
    }

    pub fn render(&self) -> String {
        self.lines.join("\n")
    }

    pub fn height(&self) -> usize {
        self.lines.len()
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn baseline(&self) -> usize {
        self.baseline
    }

    pub fn pad_to(&self, target_width: usize) -> Block {
        if target_width <= self.width {
            return self.clone();
        }
        let lines: Vec<String> = self
            .lines
            .iter()
            .map(|l| pad_right(l, target_width))
            .collect();
        Block {
            lines,
            width: target_width,
            baseline: self.baseline,
        }
    }

    pub fn center(&self, target_width: usize) -> Block {
        if self.lines.is_empty() || target_width <= self.width {
            return self.clone();
        }
        let lines: Vec<String> = self
            .lines
            .iter()
            .map(|l| center_text(l, target_width))
            .collect();
        Block {
            lines,
            width: target_width,
            baseline: self.baseline,
        }
    }

    /// Vertical concatenation. The narrower block is padded to match width.
    pub fn above(&self, other: &Block) -> Block {
        let new_w = self.width.max(other.width);
        let self_lines: Vec<String> = self.lines.iter().map(|l| pad_right(l, new_w)).collect();
        let other_lines: Vec<String> = other.lines.iter().map(|l| pad_right(l, new_w)).collect();
        let mut lines = self_lines;
        lines.extend(other_lines);
        Block {
            lines,
            width: new_w,
            baseline: self.baseline,
        }
    }

    /// Horizontal concatenation, aligning blocks at their baseline positions.
    pub fn beside(&self, other: &Block) -> Block {
        let self_h = self.lines.len();
        let other_h = other.lines.len();
        let self_bl = self.baseline;
        let other_bl = other.baseline;
        let diff = self_bl.abs_diff(other_bl);
        let top_pad_left = if self_bl < other_bl { diff } else { 0 };
        let top_pad_right = if other_bl < self_bl { diff } else { 0 };
        let result_h = (top_pad_left + self_h).max(top_pad_right + other_h);
        let self_padded = vpad_at(self, result_h, top_pad_left);
        let other_padded = vpad_at(other, result_h, top_pad_right);
        let new_w = self.width + other.width;
        let result_bl = self_bl.max(other_bl);
        let lines: Vec<String> = (0..result_h)
            .map(|i| self_padded[i].clone() + &other_padded[i])
            .collect();
        Block {
            lines,
            width: new_w,
            baseline: result_bl,
        }
    }

    pub fn append_left(&self, label: &Block) -> Block {
        label.beside(self)
    }

    pub fn append_right(&self, label: &Block) -> Block {
        self.beside(label)
    }

    /// Enclose every line with `left` and `right` delimiters.
    pub fn enclose(&self, left: &str, right: &str) -> Block {
        if self.lines.is_empty() {
            return Block::from(&format!("{left}{right}"));
        }
        let lines: Vec<String> = self
            .lines
            .iter()
            .map(|l| format!("{left}{l}{right}"))
            .collect();
        Block {
            lines,
            width: self.width + display_width(left) + display_width(right),
            baseline: self.baseline,
        }
    }

    /// Build a single-row block from cell text strings, each padded to the
    /// corresponding column width, then joined by `gap`.
    pub fn row(cells: &[String], widths: &[usize], gap: &str) -> Block {
        let joined: Vec<String> = cells
            .iter()
            .enumerate()
            .map(|(j, c)| pad_right(c, widths[j]))
            .collect();
        Block::from(&joined.join(gap))
    }
}

pub fn pad_right(s: &str, width: usize) -> String {
    let dw = display_width(s);
    if width <= dw {
        return s.to_string();
    }
    format!("{s}{}", " ".repeat(width - dw))
}

/// Compute the maximum width per column across all rows of cell strings.
#[allow(dead_code)]
pub fn column_widths(cells: &[Vec<String>]) -> Vec<usize> {
    column_max_widths(cells, |s| display_width(s))
}

/// Compute the maximum width per column of any row-shaped table, where each
/// cell's width comes from `width_of`.
pub fn column_max_widths<W>(rows: &[Vec<W>], width_of: impl Fn(&W) -> usize) -> Vec<usize> {
    let max_cols = rows.iter().map(Vec::len).max().unwrap_or(0);
    let mut widths = vec![0usize; max_cols];
    for row in rows {
        for (j, cell) in row.iter().enumerate() {
            let w = width_of(cell);
            if w > widths[j] {
                widths[j] = w;
            }
        }
    }
    widths
}

pub(crate) fn center_text(s: &str, width: usize) -> String {
    let dw = display_width(s);
    if width <= dw {
        return s.to_string();
    }
    let need = width - dw;
    let left = need / 2;
    let right = need - left;
    format!("{}{s}{}", " ".repeat(left), " ".repeat(right))
}

/// Terminal display width (in columns) of a string, counting full-width
/// characters as 2 and combining/zero-width marks as 0. Unlike `len()`, this
/// is based on Unicode code points, so astral-plane characters (e.g. math
/// alphanumerics like 𝟙) count as their on-screen width of 1.
pub fn display_width(s: &str) -> usize {
    s.chars().map(char_width).sum()
}

fn char_width(c: char) -> usize {
    let code = c as u32;
    if code == 0 {
        return 0;
    }
    if is_zero_width_mark(code) {
        0
    } else if is_east_asian_wide(code) {
        2
    } else {
        1
    }
}

/// Vertically pad a block so it has exactly `target` rows, inserting blank
/// lines at the top so the block's content starts at row `top_offset`.
fn vpad_at(b: &Block, target: usize, top_offset: usize) -> Vec<String> {
    let h = b.lines.len();
    if h >= target {
        return b.lines.clone();
    }
    let empty = " ".repeat(b.width);
    let mut top = vec![empty.clone(); top_offset];
    let bottom = vec![empty; target.saturating_sub(top_offset.saturating_add(h))];
    top.extend(b.lines.clone());
    top.extend(bottom);
    top
}
