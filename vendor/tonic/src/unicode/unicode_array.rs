use crate::ast::{ArrayColumn, ColumnSeparationType, ParseNode};
use crate::unicode::block::{Block, center_text, column_max_widths, display_width, pad_right};
use crate::unicode::config::{LineStyle, RenderState};
use crate::unicode::unicode::{content_block, render_node_internal};

pub(crate) fn render_array_inline(body: &[Vec<ParseNode>], state: &RenderState) -> String {
    body.iter()
        .map(|r| render_row(r, state))
        .collect::<Vec<_>>()
        .join("; ")
}

fn render_row(row: &[ParseNode], state: &RenderState) -> String {
    row.iter()
        .map(|c| render_node_internal(c, state))
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn render_array_block(
    body: &[Vec<ParseNode>],
    columns: Option<&[ArrayColumn]>,
    hlines_before_row: &[Vec<bool>],
    separation: Option<ColumnSeparationType>,
    state: &RenderState,
) -> String {
    cell_block(body, columns, hlines_before_row, separation, state).render()
}

pub(crate) fn render_leftright_block(
    left: &str,
    right: &str,
    body: &[Vec<ParseNode>],
    columns: Option<&[ArrayColumn]>,
    hlines_before_row: &[Vec<bool>],
    separation: Option<ColumnSeparationType>,
    state: &RenderState,
) -> Block {
    cell_block(body, columns, hlines_before_row, separation, state).enclose(left, right)
}

pub(crate) fn cell_block(
    body: &[Vec<ParseNode>],
    columns: Option<&[ArrayColumn]>,
    hlines_before_row: &[Vec<bool>],
    separation: Option<ColumnSeparationType>,
    state: &RenderState,
) -> Block {
    let cell_blocks: Vec<Vec<Block>> = body
        .iter()
        .map(|r| r.iter().map(|c| content_block(c, state)).collect())
        .collect();
    let n = cell_blocks.len();
    let mut last = n.wrapping_sub(1);
    while last < n && cell_blocks[last].iter().all(|b| b.lines.is_empty()) {
        last = last.wrapping_sub(1);
    }
    if last >= n {
        return Block::empty();
    }
    let rows: Vec<Vec<Block>> = cell_blocks[..=last].to_vec();
    let widths = column_block_widths(&rows);
    let ncols = widths.len();
    if ncols == 0 {
        return Block::empty();
    }
    let aligns = column_alignments(columns, ncols, separation);
    let vlines = separator_glyphs(columns, ncols, state.config.line_style);
    let (line_width, vbar_pos) = row_geometry(&widths, &vlines, separation);
    let hline_kind = hline_band_kinds(hlines_before_row, last);
    let lines = render_array_lines(
        &rows
            .iter()
            .map(|row| row_block(row, &widths, &aligns, &vlines, separation))
            .collect::<Vec<_>>(),
        hlines_before_row,
        &vbar_pos,
        line_width,
        &hline_kind,
        state.config.line_style,
        last,
    );
    let block = Block::from(&lines.join("\n"));
    let block = pad_gather(&block, separation);
    let mut block = block;
    block.baseline = block.height() / 2;
    block
}

/// For `gather`/`gathered`, KaTeX centers each row across the display width.
fn pad_gather(block: &Block, separation: Option<ColumnSeparationType>) -> Block {
    if separation != Some(ColumnSeparationType::GatherSeparation) {
        return block.clone();
    }
    let lines: Vec<String> = block.lines.iter().map(|l| format!("    {l}    ")).collect();
    Block {
        lines,
        width: block.width + 8,
        baseline: block.baseline,
    }
}

/// Computes the display width of every column from the maximum block width in
/// that column across all rows.
fn column_block_widths(rows: &[Vec<Block>]) -> Vec<usize> {
    column_max_widths(rows, super::block::Block::width)
}

fn column_alignments(
    columns: Option<&[ArrayColumn]>,
    ncols: usize,
    separation: Option<ColumnSeparationType>,
) -> Vec<String> {
    let mut parsed: Vec<String> = vec!["l".to_string(); ncols];
    if let Some(cols) = columns {
        let mut col = 0;
        for spec in cols {
            if let ArrayColumn::AlignColumn { alignment, .. } = spec {
                if col < ncols {
                    parsed[col] = alignment.clone();
                }
                col += 1;
            }
        }
    }
    match separation {
        Some(
            ColumnSeparationType::AlignSeparation
            | ColumnSeparationType::AlignAtSeparation
            | ColumnSeparationType::GatherSeparation,
        ) => parsed,
        _ => vec!["l".to_string(); ncols],
    }
}

#[allow(clippy::needless_range_loop)]
fn row_block(
    cells: &[Block],
    widths: &[usize],
    aligns: &[String],
    vlines: &[Option<String>],
    separation: Option<ColumnSeparationType>,
) -> Block {
    let ncols = widths.len();
    let padded: Vec<Block> = (0..ncols)
        .map(|j| {
            pad_cell(
                cells.get(j).unwrap_or(&Block::empty()),
                widths[j],
                &aligns[j],
            )
        })
        .collect();
    let mut row_bl = 0;
    for j in 0..ncols {
        row_bl = row_bl.max(padded[j].baseline);
    }
    let tops: Vec<usize> = (0..ncols).map(|j| row_bl - padded[j].baseline).collect();
    let mut row_h = row_bl + 1;
    for j in 0..ncols {
        row_h = row_h.max(tops[j] + padded[j].lines.len());
    }
    let mut lines: Vec<String> = vec![String::new(); row_h];
    let mut width = 0;
    if let Some(v) = &vlines[0] {
        for i in 0..row_h {
            lines[i].push_str(v);
        }
        width += display_width(v);
    }
    for j in 0..ncols {
        let cell_lines = vertical_lines(&padded[j], tops[j], row_h);
        for i in 0..row_h {
            lines[i].push_str(&cell_lines[i]);
        }
        width += widths[j];
        if j < ncols - 1 {
            let gap = boundary_gap(vlines, j + 1, separation);
            for i in 0..row_h {
                lines[i].push_str(&gap);
            }
            width += display_width(&gap);
        } else if let Some(v) = &vlines[ncols] {
            for i in 0..row_h {
                lines[i].push_str(v);
            }
            width += display_width(v);
        }
    }
    Block {
        lines,
        width,
        baseline: row_bl,
    }
}

fn pad_cell(b: &Block, width: usize, align: &str) -> Block {
    if b.lines.is_empty() {
        return Block {
            lines: Vec::new(),
            width,
            baseline: 0,
        };
    }
    if width <= b.width {
        return b.clone();
    }
    match align {
        "r" => Block {
            lines: b
                .lines
                .iter()
                .map(|l| format!("{}{l}", " ".repeat(width - display_width(l))))
                .collect(),
            width,
            baseline: b.baseline,
        },
        "c" => Block {
            lines: b.lines.iter().map(|l| center_text(l, width)).collect(),
            width,
            baseline: b.baseline,
        },
        _ => Block {
            lines: b.lines.iter().map(|l| pad_right(l, width)).collect(),
            width,
            baseline: b.baseline,
        },
    }
}

fn vertical_lines(b: &Block, top: usize, row_h: usize) -> Vec<String> {
    let h = b.lines.len();
    let empty = " ".repeat(b.width);
    let mut top_lines = vec![empty.clone(); top];
    let bottom_lines = vec![empty; row_h.saturating_sub(top.saturating_add(h))];
    top_lines.extend(b.lines.clone());
    top_lines.extend(bottom_lines);
    top_lines
}

fn column_gap(separation: Option<ColumnSeparationType>, boundary: usize) -> String {
    match separation {
        Some(ColumnSeparationType::AlignSeparation) => {
            if boundary.is_multiple_of(2) {
                "  ".to_string()
            } else {
                String::new()
            }
        }
        Some(
            ColumnSeparationType::AlignAtSeparation
            | ColumnSeparationType::GatherSeparation
            | ColumnSeparationType::CdSeparation,
        ) => String::new(),
        Some(ColumnSeparationType::SmallSeparation) => "  ".to_string(),
        None => "  ".to_string(),
    }
}

fn boundary_gap(
    vlines: &[Option<String>],
    boundary: usize,
    separation: Option<ColumnSeparationType>,
) -> String {
    match &vlines[boundary] {
        Some(v) => format!(" {v} "),
        None => column_gap(separation, boundary),
    }
}

#[allow(clippy::needless_range_loop)]
fn render_array_lines(
    row_blocks: &[Block],
    hlines_before_row: &[Vec<bool>],
    vbar_pos: &[Option<usize>],
    line_width: usize,
    hline_kind: &[Vec<usize>],
    style: LineStyle,
    last: usize,
) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for k in 0..=last {
        if let Some(bands) = hlines_before_row.get(k) {
            for (band, dashed) in bands.iter().enumerate() {
                lines.push(hline_rule(
                    line_width,
                    vbar_pos,
                    style,
                    *dashed,
                    band_of(hline_kind, k, band),
                ));
            }
        }
        lines.push(row_blocks[k].render());
    }
    for k in (last + 1)..hlines_before_row.len() {
        if let Some(bands) = hlines_before_row.get(k) {
            for (band, dashed) in bands.iter().enumerate() {
                lines.push(hline_rule(
                    line_width,
                    vbar_pos,
                    style,
                    *dashed,
                    band_of(hline_kind, k, band),
                ));
            }
        }
    }
    lines
}

fn hline_band_kinds(hlines_before_row: &[Vec<bool>], last: usize) -> Vec<Vec<usize>> {
    let mut kinds: Vec<Vec<usize>> = Vec::new();
    for k in 0..hlines_before_row.len() {
        let kind = if k == 0 {
            0
        } else if k <= last {
            1
        } else {
            2
        };
        kinds.push(
            hlines_before_row
                .get(k)
                .unwrap_or(&Vec::new())
                .iter()
                .map(|_| kind)
                .collect(),
        );
    }
    kinds
}

fn band_of(kinds: &[Vec<usize>], k: usize, band: usize) -> usize {
    kinds
        .get(k)
        .and_then(|arr| arr.get(band))
        .copied()
        .unwrap_or(1)
}

fn row_geometry(
    widths: &[usize],
    vlines: &[Option<String>],
    separation: Option<ColumnSeparationType>,
) -> (usize, Vec<Option<usize>>) {
    let n = widths.len();
    let mut vbar_pos: Vec<Option<usize>> = vec![None; n + 1];
    let mut cursor = 0;
    if vlines[0].is_some() {
        vbar_pos[0] = Some(0);
        cursor = 1;
    }
    for j in 0..n {
        cursor += widths[j];
        if j < n - 1 {
            let gap = boundary_gap(vlines, j + 1, separation);
            if vlines[j + 1].is_some() {
                vbar_pos[j + 1] = Some(cursor + 1);
                cursor += display_width(&gap);
            } else {
                cursor += display_width(&gap);
            }
        } else if vlines[n].is_some() {
            vbar_pos[n] = Some(cursor);
            cursor += 1;
        }
    }
    (cursor, vbar_pos)
}

fn separator_glyphs(
    columns: Option<&[ArrayColumn]>,
    ncols: usize,
    style: LineStyle,
) -> Vec<Option<String>> {
    let mut result: Vec<Option<String>> = vec![None; ncols + 1];
    let Some(cols) = columns else {
        return result;
    };
    let mut boundary = 0;
    for col in cols {
        match col {
            ArrayColumn::AlignColumn { .. } => boundary += 1,
            ArrayColumn::SeparatorColumn { separator } => {
                if boundary <= ncols {
                    result[boundary] = Some(array_vbar(style, separator == ":"));
                }
            }
        }
    }
    result
}

fn array_vbar(style: LineStyle, dashed: bool) -> String {
    match style {
        LineStyle::Ascii => {
            if dashed {
                ":".to_string()
            } else {
                "|".to_string()
            }
        }
        LineStyle::Unicode => {
            if dashed {
                "┊".to_string()
            } else {
                "│".to_string()
            }
        }
    }
}

fn hline_rule(
    width: usize,
    vbar_pos: &[Option<usize>],
    style: LineStyle,
    dashed: bool,
    kind: usize,
) -> String {
    let hbar = match style {
        LineStyle::Ascii => "-",
        LineStyle::Unicode => {
            if dashed {
                "┄"
            } else {
                "─"
            }
        }
    };
    let left_pos = vbar_pos[0];
    let right_pos = vbar_pos[vbar_pos.len() - 1];
    let mut result = String::new();
    for i in 0..width {
        let junction = if is_vbar_position(vbar_pos, i) {
            hline_junction(style, kind, left_pos, right_pos, i)
        } else {
            hbar.to_string()
        };
        result.push_str(&junction);
    }
    result
}

fn is_vbar_position(vbar_pos: &[Option<usize>], i: usize) -> bool {
    vbar_pos.contains(&Some(i))
}

fn hline_junction(
    style: LineStyle,
    kind: usize,
    left_pos: Option<usize>,
    right_pos: Option<usize>,
    i: usize,
) -> String {
    match style {
        LineStyle::Ascii => "+".to_string(),
        LineStyle::Unicode => match kind {
            0 => {
                if left_pos == Some(i) {
                    "┌".to_string()
                } else if right_pos == Some(i) {
                    "┐".to_string()
                } else {
                    "┬".to_string()
                }
            }
            2 => {
                if left_pos == Some(i) {
                    "└".to_string()
                } else if right_pos == Some(i) {
                    "┘".to_string()
                } else {
                    "┴".to_string()
                }
            }
            _ => {
                if left_pos == Some(i) {
                    "├".to_string()
                } else if right_pos == Some(i) {
                    "┤".to_string()
                } else {
                    "┼".to_string()
                }
            }
        },
    }
}
