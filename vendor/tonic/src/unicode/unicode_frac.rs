use crate::ast::{ParseNode, StyleLevel};
use crate::unicode::block::Block;
use crate::unicode::config::{RenderState, line_style_frac_bar};
use crate::unicode::unicode::{render_delimiter, render_node_internal};

use super::block::{center_text, display_width};

/// Renders a stacked fraction block. When `has_bar_line` is false (e.g.
/// `\atop`/`\atopfrac`) the numerator is stacked directly above the
/// denominator with no fraction bar.
pub(crate) fn frac_block(
    numer: &ParseNode,
    denom: &ParseNode,
    state: &RenderState,
    has_bar_line: bool,
) -> Block {
    let child_tight = state.style != StyleLevel::DisplayStyle;
    let num_state = RenderState {
        in_display: false,
        in_tight: child_tight,
        ..state.clone()
    };
    let den_state = RenderState {
        in_display: false,
        in_tight: child_tight,
        ..state.clone()
    };
    let num = Block::from(&render_node_internal(numer, &num_state));
    let den = Block::from(&render_node_internal(denom, &den_state));
    let bar_w = num.width().max(den.width()) + 2;
    let frac = if has_bar_line {
        let bar = Block::from(&line_style_frac_bar(state.config.line_style).repeat(bar_w));
        num.center(bar_w).above(&bar).above(&den.center(bar_w))
    } else {
        num.center(bar_w).above(&den.center(bar_w))
    };
    let mut frac = frac;
    frac.baseline = num.height();
    frac
}

pub(crate) fn render_genfrac_block(
    numer: &ParseNode,
    denom: &ParseNode,
    state: &RenderState,
    has_bar_line: bool,
) -> String {
    frac_block(numer, denom, state, has_bar_line).render()
}

/// Wrap a block with single-line delimiters aligned at its baseline row, so
/// that a delimiter beside a block fraction sits exactly on the fraction bar.
pub(crate) fn wrap_delims(block: &Block, left: Option<&str>, right: Option<&str>) -> Block {
    let l = left.map_or_else(Block::empty, |d| Block::from(&render_delimiter(d)));
    let r = right.map_or_else(Block::empty, |d| Block::from(&render_delimiter(d)));
    l.beside(block).beside(&r)
}

/// Renders a bar-less delimited stack (binomial, `\bracefrac`, `\brackfrac`)
/// as a block whose delimiters span both rows. Used in display mode; inline
/// contexts render as `(n,k)` instead.
pub(crate) fn barless_delimited_block(
    numer: &ParseNode,
    denom: &ParseNode,
    left: Option<&str>,
    right: Option<&str>,
    state: &RenderState,
) -> Block {
    let child_tight = state.style != StyleLevel::DisplayStyle;
    let num_state = RenderState {
        in_display: false,
        in_tight: child_tight,
        ..state.clone()
    };
    let den_state = RenderState {
        in_display: false,
        in_tight: child_tight,
        ..state.clone()
    };
    let num = Block::from(&render_node_internal(numer, &num_state));
    let den = Block::from(&render_node_internal(denom, &den_state));
    let w = num.width().max(den.width());
    let l = left.map(render_delimiter).unwrap_or_default();
    let r = right.map(render_delimiter).unwrap_or_default();
    let lines = vec![
        format!("{l}{}{r}", center_text(&num.render(), w)),
        format!("{l}{}{r}", center_text(&den.render(), w)),
    ];
    Block {
        lines,
        width: w + display_width(&l) + display_width(&r),
        baseline: 0,
    }
}
