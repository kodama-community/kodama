use crate::ast::ParseNode;
use crate::unicode::block::{Block, center_text, pad_right};
use crate::unicode::config::{LineStyle, RenderState, line_style_frac_bar};
use crate::unicode::unicode::render_node_internal;

/// Cell classification for commutative-diagram (`CD`) rendering.
#[allow(clippy::enum_variant_names)]
#[derive(Clone)]
enum CdCell {
    CdEmpty,
    CdNode(String),
    CdHArrow(String, String, String),
    CdVArrow(String, String, String),
}

/// Renders a `CD` array as a block commutative diagram. Even rows hold the
/// nodes and horizontal arrows; odd rows hold the vertical arrows connecting
/// them.
pub(crate) fn render_cd_block(body: &[Vec<ParseNode>], state: &RenderState) -> Block {
    let mut grid: Vec<Vec<CdCell>> = body
        .iter()
        .map(|row| row.iter().map(|c| classify_cd_cell(c, state)).collect())
        .collect();
    // The parser keeps KaTeX's terminal structural row for HTML layout.
    // It has no cell content to display in the plain-text diagram.
    if grid.last().is_some_and(Vec::is_empty) {
        grid.pop();
    }
    if grid.is_empty() || grid[0].is_empty() {
        return Block::empty();
    }
    let n_nodes = grid[0].len() / 2 + 1;
    let hbar = line_style_frac_bar(state.config.line_style);
    let vbar = cd_vbar(state.config.line_style);
    let (node_w, arrow_w, offsets) = cd_column_widths(&grid, n_nodes, &vbar);
    let mut bands: Vec<Block> = Vec::new();
    for (row_index, row) in grid.iter().enumerate() {
        if row_index % 2 == 0 {
            bands.push(cd_node_row_block(row, &node_w, &arrow_w, &offsets, &hbar));
        } else {
            bands.push(cd_varrow_row_block(row, &node_w, &arrow_w, &offsets, &vbar));
        }
    }
    let mut result = Block::empty();
    for band in &bands {
        if result.height() == 0 {
            result = band.clone();
        } else {
            result = result.above(band);
        }
    }
    result.baseline = result.height() / 2;
    result
}

fn classify_cd_cell(cell: &ParseNode, state: &RenderState) -> CdCell {
    match cell {
        ParseNode::Styling { body, .. } if body.len() == 1 => {
            classify_cd_inner(&body[0], cell, state)
        }
        ParseNode::Styling { body, .. } if body.is_empty() => CdCell::CdEmpty,
        _ => CdCell::CdNode(render_node_internal(cell, state)),
    }
}

fn classify_cd_inner(inner: &ParseNode, cell: &ParseNode, state: &RenderState) -> CdCell {
    match inner {
        ParseNode::XArrow {
            label, body, below, ..
        } if cd_arrow_dir(label).is_some() => {
            let dir = cd_arrow_dir(label).unwrap();
            CdCell::CdHArrow(
                dir,
                render_node_internal(body, state),
                below
                    .as_ref()
                    .map(|b| render_node_internal(b, state))
                    .unwrap_or_default(),
            )
        }
        ParseNode::CdParent { fragment, .. } => classify_cd_parent(fragment, cell, state),
        _ => CdCell::CdNode(render_node_internal(cell, state)),
    }
}

fn classify_cd_parent(fragment: &ParseNode, cell: &ParseNode, state: &RenderState) -> CdCell {
    match fragment {
        ParseNode::OrdGroup { body, .. } => {
            let mut left = String::new();
            let mut right = String::new();
            let mut dir = "V".to_string();
            for node in body {
                match node {
                    ParseNode::CdLabel { side, label, .. } if side == "left" => {
                        left = render_node_internal(label, state);
                    }
                    ParseNode::CdLabel { side, label, .. } if side == "right" => {
                        right = render_node_internal(label, state);
                    }
                    ParseNode::DelimSizing { delim, .. } if delim == "\\uparrow" => {
                        dir = "A".to_string();
                    }
                    _ => (),
                }
            }
            CdCell::CdVArrow(dir, left, right)
        }
        _ => CdCell::CdNode(render_node_internal(cell, state)),
    }
}

fn cd_arrow_dir(label: &str) -> Option<String> {
    match label {
        "\\\\cdrightarrow" => Some(">".to_string()),
        "\\\\cdleftarrow" => Some("<".to_string()),
        "\\\\cdlongequal" => Some("=".to_string()),
        _ => None,
    }
}

fn cd_vbar(style: LineStyle) -> String {
    match style {
        LineStyle::Ascii => "|".to_string(),
        LineStyle::Unicode => "│".to_string(),
    }
}

fn cd_harrow_line(dir: &str, w: usize, hbar: &str) -> String {
    match dir {
        ">" => format!("{hbar}→", hbar = hbar.repeat(w - 1)),
        "<" => format!("←{hbar}", hbar = hbar.repeat(w - 1)),
        _ => "=".repeat(w),
    }
}

fn cd_varrow_lines(dir: &str, left: &str, right: &str, vbar: &str, offset: usize) -> Vec<String> {
    let pad = " ".repeat(offset);
    let labels = format!("{}{vbar}{right}", pad_right(left, offset));
    match dir {
        "V" => vec![format!("{pad}{vbar}"), labels, format!("{pad}↓")],
        "A" => vec![format!("{pad}↑"), labels, format!("{pad}{vbar}")],
        _ => vec![format!("{pad}{vbar}"), labels, format!("{pad}{vbar}")],
    }
}

fn cd_column_widths(
    grid: &[Vec<CdCell>],
    n_nodes: usize,
    vbar: &str,
) -> (Vec<usize>, Vec<usize>, Vec<usize>) {
    let offsets = cd_shaft_offsets(grid, n_nodes);
    let (node_w, mut arrow_w) = cd_cell_widths(grid, n_nodes, &offsets, vbar);
    for (k, w) in arrow_w.iter_mut().enumerate() {
        *w += offsets[k + 1];
    }
    (node_w, arrow_w, offsets)
}

/// Pass 1: the shaft offset of each node column, i.e. the widest left label
/// among the vertical arrows in that column.
fn cd_shaft_offsets(grid: &[Vec<CdCell>], n_nodes: usize) -> Vec<usize> {
    let mut offsets = vec![0usize; n_nodes];
    for (row_index, row) in grid.iter().enumerate() {
        if row_index % 2 != 1 {
            continue;
        }
        for (col, cell) in row.iter().enumerate() {
            if col % 2 != 0 {
                continue;
            }
            if let CdCell::CdVArrow(_, left, _) = cell {
                offsets[col / 2] = offsets[col / 2].max(left.chars().count());
            }
        }
    }
    offsets
}

/// Pass 2: node widths from node text (shifted by the shaft offset) and
/// v-arrow lines; arrow widths from h-arrow labels and cell node text.
fn cd_cell_widths(
    grid: &[Vec<CdCell>],
    n_nodes: usize,
    offsets: &[usize],
    vbar: &str,
) -> (Vec<usize>, Vec<usize>) {
    let mut node_w = vec![0usize; n_nodes];
    let mut arrow_w = vec![4usize; n_nodes.saturating_sub(1)];
    for (row_index, row) in grid.iter().enumerate() {
        if row_index % 2 == 0 {
            for (col, cell) in row.iter().enumerate() {
                if col % 2 == 0 {
                    if let CdCell::CdNode(text) = cell {
                        node_w[col / 2] =
                            node_w[col / 2].max(offsets[col / 2] + text.chars().count());
                    }
                } else if let CdCell::CdHArrow(_, upper, lower) = cell {
                    arrow_w[col / 2] = arrow_w[col / 2]
                        .max(upper.chars().count())
                        .max(lower.chars().count());
                } else if let CdCell::CdNode(text) = cell {
                    arrow_w[col / 2] = arrow_w[col / 2].max(text.chars().count());
                }
            }
        } else {
            for (col, cell) in row.iter().enumerate() {
                if col % 2 != 0 {
                    continue;
                }
                if let CdCell::CdVArrow(dir, left, right) = cell {
                    for line in cd_varrow_lines(dir, left, right, vbar, offsets[col / 2]) {
                        node_w[col / 2] = node_w[col / 2].max(line.chars().count());
                    }
                }
            }
        }
    }
    (node_w, arrow_w)
}

fn cd_node_row_block(
    row: &[CdCell],
    node_w: &[usize],
    arrow_w: &[usize],
    offsets: &[usize],
    hbar: &str,
) -> Block {
    let n = row.len();
    let mut main: Vec<String> = vec![String::new(); n];
    for (col, cell) in row.iter().enumerate() {
        if col % 2 == 0 {
            main[col] = match cell {
                CdCell::CdNode(text) => format!("{}{text}", " ".repeat(offsets[col / 2])),
                _ => String::new(),
            };
        } else {
            main[col] = match cell {
                CdCell::CdHArrow(dir, _, _) => cd_harrow_line(dir, arrow_w[col / 2], hbar),
                CdCell::CdNode(text) => text.clone(),
                _ => String::new(),
            };
        }
    }
    let mut lines = vec![cd_pad_line(&main, node_w, arrow_w, offsets)];
    if cd_row_has_upper(row) {
        let mut cells: Vec<String> = vec![String::new(); n];
        for (col, cell) in row.iter().enumerate() {
            if col % 2 == 1 {
                if let CdCell::CdHArrow(_, upper, _) = cell {
                    cells[col] = center_text(upper, arrow_w[col / 2]);
                }
            }
        }
        lines.insert(0, cd_pad_line(&cells, node_w, arrow_w, offsets));
    }
    if cd_row_has_lower(row) {
        let mut cells: Vec<String> = vec![String::new(); n];
        for (col, cell) in row.iter().enumerate() {
            if col % 2 == 1 {
                if let CdCell::CdHArrow(_, _, lower) = cell {
                    cells[col] = center_text(lower, arrow_w[col / 2]);
                }
            }
        }
        lines.push(cd_pad_line(&cells, node_w, arrow_w, offsets));
    }
    Block::from(&lines.join("\n"))
}

fn cd_row_has_upper(row: &[CdCell]) -> bool {
    row.iter()
        .any(|cell| matches!(cell, CdCell::CdHArrow(_, upper, _) if !upper.is_empty()))
}

fn cd_row_has_lower(row: &[CdCell]) -> bool {
    row.iter()
        .any(|cell| matches!(cell, CdCell::CdHArrow(_, _, lower) if !lower.is_empty()))
}

fn cd_varrow_row_block(
    row: &[CdCell],
    node_w: &[usize],
    arrow_w: &[usize],
    offsets: &[usize],
    vbar: &str,
) -> Block {
    let n = row.len();
    let mut lines: Vec<String> = Vec::new();
    for h in 0..3 {
        let mut cells: Vec<String> = vec![String::new(); n];
        for (col, cell) in row.iter().enumerate() {
            if col % 2 == 0 {
                if let CdCell::CdVArrow(dir, left, right) = cell {
                    cells[col] = cd_varrow_lines(dir, left, right, vbar, offsets[col / 2])[h].clone();
                }
            }
        }
        lines.push(cd_pad_line(&cells, node_w, arrow_w, offsets));
    }
    Block::from(&lines.join("\n"))
}

fn cd_pad_line(cells: &[String], node_w: &[usize], arrow_w: &[usize], offsets: &[usize]) -> String {
    let mut result = String::new();
    let mut j = 0;
    while j < cells.len() {
        if j % 2 == 0 {
            result.push_str(&pad_right(&cells[j], node_w[j / 2]));
            if j + 1 < cells.len() {
                result.push(' ');
            }
        } else {
            result.push_str(&pad_right(&cells[j], arrow_w[j / 2]));
            let next_node = j / 2 + 1;
            if next_node < offsets.len() {
                let gap = 1usize.saturating_sub(offsets[next_node]);
                result.push_str(&" ".repeat(gap));
            }
        }
        j += 1;
    }
    result
}
