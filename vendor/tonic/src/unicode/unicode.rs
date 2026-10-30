use std::rc::Rc;

use crate::anvil::{
    atom_family_name, cancel_bin_atoms, command_name, em_value, is_null_delimiter,
    join_with_spacing, math_choice_variant, math_spacing, resolve_symbol, SpacableItem,
};
use crate::ast::{ColumnSeparationType, Measurement, OperatorContent, ParseNode, StyleLevel};
use crate::unicode::block::center_text;
use crate::unicode::block::{display_width, Block};
use crate::unicode::config::{LineStyle, RenderConfig, RenderState};
use crate::unicode::unicode_array::{
    cell_block, render_array_block, render_array_inline, render_leftright_block,
};
use crate::unicode::unicode_cd::render_cd_block;
use crate::unicode::unicode_frac::{
    barless_delimited_block, frac_block, render_genfrac_block, wrap_delims,
};
use crate::unicode_font::unicode_font_character;
use crate::unicode_scripts::{unicode_script_character, UnicodeScriptKind};

use super::atomic::is_atomic_expression;

/// Renders parsed KaTeX nodes as Unicode text. Nodes without a direct Unicode
/// rendering use UnicodeMath-style function syntax so that their structure is
/// retained in plain text.
pub fn render(nodes: &[ParseNode], config: RenderConfig) -> String {
    render_internal(
        nodes,
        &RenderState {
            style: StyleLevel::TextStyle,
            in_display: false,
            in_tight: false,
            config: Rc::new(config),
        },
    )
}

fn render_internal(nodes: &[ParseNode], state: &RenderState) -> String {
    render_internal_block(nodes, state).render()
}

fn render_internal_block(nodes: &[ParseNode], state: &RenderState) -> Block {
    let items = merge_not_overlay(collect_spacable_items(nodes, state));
    let cancelled = cancel_bin_atoms(items);
    if cancelled.iter().any(|it| it.text.contains('\n')) {
        join_with_block(&cancelled, state.in_tight, &state.config.spacing)
    } else {
        Block::from(&join_with_spacing(
            &cancelled,
            state.in_tight,
            &state.config.spacing,
        ))
    }
}

fn join_with_block(items: &[SpacableItem], tight: bool, spec: &crate::anvil::SpacingSpec) -> Block {
    let mut result = item_to_block(&items[0]);
    for i in 1..items.len() {
        let prev = &items[i - 1];
        let curr = &items[i];
        if let (Some(left), Some(right)) = (&prev.atom_type, &curr.atom_type) {
            if let Some(space) = math_spacing(left, right, tight, spec) {
                result = result.beside(&Block::from(&space));
            }
        }
        result = result.beside(&item_to_block(curr));
    }
    result
}

fn item_to_block(item: &SpacableItem) -> Block {
    let mut b = Block::from(&item.text);
    b.baseline = item.baseline;
    b
}

pub(crate) fn render_node_internal(node: &ParseNode, state: &RenderState) -> String {
    let slash = &state.config.fraction_slash;
    match node {
        ParseNode::Unsupported { fallback, .. } => render_node_internal(fallback, state),
        ParseNode::Internal { .. } | ParseNode::EnvironmentEnd { .. } => String::new(),
        ParseNode::Raw { string, .. } => resolve_symbol(string),
        ParseNode::ColorToken { color, .. } => resolve_symbol(color),
        ParseNode::TextOrd { text, .. }
        | ParseNode::MathOrd { text, .. }
        | ParseNode::Spacing { text, .. }
        | ParseNode::AccentToken { text, .. }
        | ParseNode::OperatorToken { text, .. }
        | ParseNode::Atom { text, .. } => resolve_symbol(text),
        ParseNode::Size { value, .. } => render_measurement(value),
        ParseNode::Url { url, .. } => url.clone(),
        ParseNode::Styling { body, style, .. } => render_internal(
            body,
            &RenderState {
                style: *style,
                in_display: state.in_display || *style == StyleLevel::DisplayStyle,
                ..state.clone()
            },
        ),
        ParseNode::Text { body, .. } => render_internal(
            body,
            &RenderState {
                in_tight: true,
                ..state.clone()
            },
        ),
        ParseNode::MClass { body, .. }
        | ParseNode::HBox { body, .. }
        | ParseNode::Sizing { body, .. }
        | ParseNode::Color { body, .. }
        | ParseNode::Href { body, .. }
        | ParseNode::Html { body, .. }
        | ParseNode::OrdGroup { body, .. } => render_internal(body, state),
        ParseNode::Sqrt {
            body, index: None, ..
        } => render_radical("", body, state),
        ParseNode::Sqrt {
            body,
            index: Some(index),
            ..
        } => render_root(index, body, state),
        ParseNode::Infix { replace_with, .. } => replace_with.clone(),
        ParseNode::GenFrac {
            numer,
            denom,
            has_bar_line: false,
            left_delim: None,
            right_delim: None,
            ..
        } => {
            if state.in_display {
                return render_genfrac_block(numer, denom, state, false);
            }
            inline_atop(numer, denom, state)
        }
        ParseNode::GenFrac {
            numer,
            denom,
            has_bar_line: false,
            left_delim,
            right_delim,
            ..
        } if state.in_display => barless_delimited_block(
            numer,
            denom,
            left_delim.as_deref(),
            right_delim.as_deref(),
            state,
        )
        .render(),
        ParseNode::GenFrac {
            numer,
            denom,
            has_bar_line: false,
            left_delim,
            right_delim,
            ..
        } => render_delimited(
            left_delim.as_deref(),
            inline_atop(numer, denom, state),
            right_delim.as_deref(),
        ),
        ParseNode::GenFrac {
            numer,
            denom,
            left_delim: None,
            right_delim: None,
            ..
        } => {
            if state.in_display {
                return render_genfrac_block(numer, denom, state, true);
            }
            inline_fraction(numer, denom, slash, state)
        }
        ParseNode::GenFrac {
            numer,
            denom,
            has_bar_line,
            left_delim,
            right_delim,
            ..
        } => {
            if state.in_display {
                wrap_delims(
                    &frac_block(numer, denom, state, *has_bar_line),
                    left_delim.as_deref(),
                    right_delim.as_deref(),
                )
                .render()
            } else {
                render_delimited(
                    left_delim.as_deref(),
                    inline_fraction(numer, denom, slash, state),
                    right_delim.as_deref(),
                )
            }
        }
        ParseNode::Font { font, body, .. } => render_font(font, body, state),
        ParseNode::Op { content, .. } => render_operator_content(content, state),
        ParseNode::OperatorName { body, .. } => render_internal(body, state),
        ParseNode::Overline { body, .. } => {
            format!("overline({})", render_node_internal(body, state))
        }
        ParseNode::Underline { body, .. } => {
            format!("underline({})", render_node_internal(body, state))
        }
        ParseNode::Smash { body, .. }
        | ParseNode::VCenter { body, .. }
        | ParseNode::RaiseBox { body, .. }
        | ParseNode::Lap { body, .. } => render_node_internal(body, state),
        // Unicode cannot mirror arbitrary glyphs; preserve the unreflected body.
        ParseNode::ReflectBox { body, .. } => render_node_internal(body, state),
        ParseNode::VPhantom { body, .. } => {
            format!("vphantom({})", render_node_internal(body, state))
        }
        ParseNode::CdParent { fragment, .. } => render_node_internal(fragment, state),
        ParseNode::Phantom { body, .. } => format!("phantom({})", render_internal(body, state)),
        ParseNode::Pmb { body, .. } => format!("bold({})", render_internal(body, state)),
        ParseNode::Rule { width, height, .. } => format!(
            "rule({},{})",
            render_measurement(width),
            render_measurement(height)
        ),
        ParseNode::MathChoice {
            display,
            text,
            script,
            scriptscript,
            ..
        } => render_internal(
            &math_choice_variant(display, text, script, scriptscript, state.style),
            state,
        ),
        ParseNode::HorizBrace { label, base, .. } => {
            format!(
                "{}({})",
                command_name(label),
                render_node_internal(base, state)
            )
        }
        ParseNode::XArrow {
            label,
            body,
            below: None,
            ..
        } => format!(
            "{}({})",
            command_name(label),
            render_node_internal(body, state)
        ),
        ParseNode::XArrow {
            label,
            body,
            below: Some(below),
            ..
        } => format!(
            "{}({},{})",
            command_name(label),
            render_node_internal(body, state),
            render_node_internal(below, state)
        ),
        ParseNode::AccentUnder { label, base, .. } => format!(
            "underaccent({},{})",
            command_name(label),
            render_node_internal(base, state)
        ),
        ParseNode::DelimSizing { delim, .. }
        | ParseNode::LeftRightRight { delim, .. }
        | ParseNode::Middle { delim, .. } => render_delimiter(delim),
        ParseNode::LeftRight {
            body, left, right, ..
        } => {
            if state.in_display {
                render_leftright_display_block(body, left, right, state).render()
            } else {
                format!(
                    "{}{}{}",
                    render_delimiter(left),
                    render_internal(body, state),
                    render_delimiter(right)
                )
            }
        }
        ParseNode::Kern { dimension, .. } => render_kern(dimension),
        ParseNode::Enclose { label, body, .. } if label == "\\fbox" => {
            render_enclose_block(body, state).render()
        }
        ParseNode::Enclose { label, body, .. } => format!(
            "{}({})",
            command_name(label),
            render_node_internal(body, state)
        ),
        ParseNode::IncludeGraphics { alt, .. } => format!("image({alt})"),
        ParseNode::Tag { body, tag, .. } => format!(
            "{}\t({})",
            render_internal(body, state),
            render_internal(tag, state)
        ),
        ParseNode::Array {
            body,
            column_separation_type: Some(ColumnSeparationType::CdSeparation),
            ..
        } => render_cd_block(body, state).render(),
        ParseNode::Array {
            body,
            columns,
            hlines_before_row,
            column_separation_type,
            ..
        } => {
            if state.in_display {
                render_array_block(
                    body,
                    columns.as_deref(),
                    hlines_before_row,
                    *column_separation_type,
                    state,
                )
            } else {
                render_array_inline(body, state)
            }
        }
        ParseNode::CdLabel { side, label, .. } => {
            format!("{side}({})", render_node_internal(label, state))
        }
        ParseNode::Cr { new_line: true, .. } => "\n".to_string(),
        ParseNode::Cr { .. } => String::new(),
        ParseNode::HtmlMathML { mathml, .. } => render_internal(mathml, state),
        ParseNode::SupSub { base, sup, sub, .. } => render_sup_sub(base, sup, sub, state),
        ParseNode::Accent { label, base, .. } => render_accent(label, base, state),
        ParseNode::Verb { body, .. } => body.clone(),
    }
}

fn merge_not_overlay(items: Vec<SpacableItem>) -> Vec<SpacableItem> {
    let mut result: Vec<SpacableItem> = Vec::new();
    let mut skip_next = false;
    let mut i = 0;
    while i < items.len() {
        if skip_next {
            skip_next = false;
            i += 1;
            continue;
        }
        let item = items[i].clone();
        if item.text == "\u{338}" && i + 1 < items.len() {
            let next = items[i + 1].clone();
            let next_text = next.text.clone();
            if next.atom_type.is_some()
                && !next_text.is_empty()
                && is_single_character_output(&next_text)
                && !crate::unicode::unicode_width::is_zero_width_mark(
                    next_text.chars().next().unwrap() as u32,
                )
            {
                result.push(SpacableItem {
                    text: format!("{next_text}\u{338}"),
                    ..next
                });
                skip_next = true;
                i += 1;
                continue;
            }
        }
        result.push(item);
        i += 1;
    }
    result
}

fn collect_spacable_items(nodes: &[ParseNode], state: &RenderState) -> Vec<SpacableItem> {
    let mut acc: Vec<SpacableItem> = Vec::new();
    for n in nodes {
        match n {
            ParseNode::Unsupported { fallback, .. } => {
                acc.extend(collect_spacable_items(
                    std::slice::from_ref(fallback.as_ref()),
                    state,
                ));
            }
            ParseNode::Styling { body, style, .. } => {
                let child_state = RenderState {
                    style: *style,
                    in_display: state.in_display || *style == StyleLevel::DisplayStyle,
                    ..state.clone()
                };
                acc.extend(collect_spacable_items(body, &child_state));
            }
            ParseNode::Color { body, .. }
            | ParseNode::Sizing { body, .. }
            | ParseNode::HBox { body, .. }
            | ParseNode::Href { body, .. }
            | ParseNode::Html { body, .. } => acc.extend(collect_spacable_items(body, state)),
            ParseNode::Spacing { text, .. } => acc.push(SpacableItem {
                atom_type: None,
                text: resolve_symbol(text),
                baseline: 0,
            }),
            _ => {
                let block = content_block(n, state);
                acc.push(SpacableItem {
                    atom_type: get_outer_atom_type(n),
                    text: block.render(),
                    baseline: block.baseline(),
                });
            }
        }
    }
    acc
}

fn get_outer_atom_type(node: &ParseNode) -> Option<String> {
    match node {
        ParseNode::Unsupported { fallback, .. } => get_outer_atom_type(fallback),
        ParseNode::Atom { family, .. } => Some(atom_family_name(*family)),
        ParseNode::MathOrd { .. }
        | ParseNode::TextOrd { .. }
        | ParseNode::Raw { .. }
        | ParseNode::ColorToken { .. }
        | ParseNode::AccentToken { .. } => Some("mord".to_string()),
        ParseNode::OperatorToken { .. } | ParseNode::Op { .. } | ParseNode::OperatorName { .. } => {
            Some("mop".to_string())
        }
        ParseNode::SupSub {
            base: Some(base), ..
        } => {
            if matches!(
                **base,
                ParseNode::Op {
                    content: OperatorContent::SymbolOperator(_),
                    ..
                }
            ) {
                Some("mbig".to_string())
            } else {
                get_outer_atom_type(base)
            }
        }
        ParseNode::SupSub { base: None, .. } => Some("mord".to_string()),
        ParseNode::Accent { base, .. } => get_outer_atom_type(base),
        ParseNode::Font { body, .. } => get_outer_atom_type(body),
        ParseNode::VCenter { body, .. }
        | ParseNode::RaiseBox { body, .. }
        | ParseNode::Lap { body, .. }
        | ParseNode::Smash { body, .. } => get_outer_atom_type(body),
        ParseNode::CdParent { fragment, .. } => get_outer_atom_type(fragment),
        ParseNode::MClass { mclass, .. } => Some(atom_family_name(*mclass)),
        ParseNode::DelimSizing { mclass, .. } => Some(atom_family_name(*mclass)),
        ParseNode::Pmb { mclass, .. } => Some(atom_family_name(*mclass)),
        ParseNode::LeftRight { .. }
        | ParseNode::LeftRightRight { .. }
        | ParseNode::Middle { .. } => Some("minner".to_string()),
        ParseNode::Spacing { .. } => None,
        ParseNode::Internal { .. } | ParseNode::EnvironmentEnd { .. } | ParseNode::Infix { .. } => {
            None
        }
        ParseNode::Color { .. }
        | ParseNode::Sizing { .. }
        | ParseNode::HBox { .. }
        | ParseNode::Href { .. }
        | ParseNode::Html { .. } => Some("mord".to_string()),
        _ => Some("mord".to_string()),
    }
}

fn render_operator_content(content: &OperatorContent, state: &RenderState) -> String {
    match content {
        OperatorContent::SymbolOperator(text) => resolve_symbol(text),
        OperatorContent::NamedOperator(text) => command_name(text),
        OperatorContent::BodyOperator(body) => render_internal(body, state),
    }
}

pub(crate) fn render_delimiter(text: &str) -> String {
    if is_null_delimiter(text) {
        return String::new();
    }
    resolve_symbol(text)
}

fn render_font(font: &str, body: &ParseNode, state: &RenderState) -> String {
    if font == "mathrm" {
        return render_node_internal(body, state);
    }
    match font_char_sequence(font, body) {
        Some(rendered) => rendered,
        None => format!("{font}({})", render_node_internal(body, state)),
    }
}

fn font_char_sequence(font: &str, node: &ParseNode) -> Option<String> {
    let mut chars: Vec<String> = Vec::new();
    if !collect_font_chars(node, &mut chars) {
        return None;
    }
    Some(render_font_text(font, &chars))
}

fn collect_font_chars(node: &ParseNode, chars: &mut Vec<String>) -> bool {
    match node {
        ParseNode::Unsupported { fallback, .. } => collect_font_chars(fallback, chars),
        ParseNode::MathOrd { text, .. }
        | ParseNode::TextOrd { text, .. }
        | ParseNode::Atom { text, .. } => {
            let rendered = resolve_symbol(text);
            chars.extend(rendered.chars().map(|ch| ch.to_string()));
            true
        }
        ParseNode::OrdGroup { body, .. } => {
            for child in body {
                if !collect_font_chars(child, chars) {
                    return false;
                }
            }
            true
        }
        _ => false,
    }
}

fn render_font_text(font: &str, chars: &[String]) -> String {
    let mut result = String::new();
    let mut failed: Vec<String> = Vec::new();
    for s in chars {
        match unicode_font_character(font, s) {
            Some(mapped) => {
                if !failed.is_empty() {
                    result.push_str(&font_fallback(font, &failed));
                    failed.clear();
                }
                result.push_str(&mapped);
            }
            None if is_font_letter(s) => failed.push(s.clone()),
            None => {
                if !failed.is_empty() {
                    result.push_str(&font_fallback(font, &failed));
                    failed.clear();
                }
                result.push_str(s);
            }
        }
    }
    if !failed.is_empty() {
        result.push_str(&font_fallback(font, &failed));
    }
    result
}

fn is_font_letter(s: &str) -> bool {
    let mut chars = s.chars();
    let Some(c) = chars.next() else {
        return false;
    };
    if chars.next().is_some() {
        return false;
    }
    c.is_ascii_uppercase() || c.is_ascii_lowercase() || ('Α'..='ω').contains(&c)
}

fn font_fallback(font: &str, chars: &[String]) -> String {
    let text: String = chars.concat();
    format!("{font}({text})")
}

fn inline_atop(numer: &ParseNode, denom: &ParseNode, state: &RenderState) -> String {
    let num_state = RenderState {
        in_tight: true,
        ..state.clone()
    };
    let den_state = RenderState {
        in_tight: true,
        ..state.clone()
    };
    format!(
        "{},{}",
        render_node_internal(numer, &num_state),
        render_node_internal(denom, &den_state)
    )
}

fn inline_fraction(
    numer: &ParseNode,
    denom: &ParseNode,
    slash: &str,
    state: &RenderState,
) -> String {
    let num_state = RenderState {
        in_tight: true,
        ..state.clone()
    };
    let den_state = RenderState {
        in_tight: true,
        ..state.clone()
    };
    format!(
        "{}{}{}",
        render_operand(numer, &num_state),
        slash,
        render_operand(denom, &den_state)
    )
}

fn render_leftright_display_block(
    body: &[ParseNode],
    left: &str,
    right: &str,
    state: &RenderState,
) -> Block {
    match body {
        [ParseNode::Array {
            body: arr_body,
            columns,
            hlines_before_row,
            column_separation_type,
            ..
        }] => render_leftright_block(
            &render_delimiter(left),
            &render_delimiter(right),
            arr_body,
            columns.as_deref(),
            hlines_before_row,
            *column_separation_type,
            state,
        ),
        _ => wrap_delims(&render_internal_block(body, state), Some(left), Some(right)),
    }
}

fn render_delimited(left: Option<&str>, content: String, right: Option<&str>) -> String {
    let left = left.map(render_delimiter).unwrap_or_default();
    let right = right.map(render_delimiter).unwrap_or_default();
    format!("{left}{content}{right}")
}

fn render_enclose_block(body: &ParseNode, state: &RenderState) -> Block {
    enclose_box(
        &render_internal_block(std::slice::from_ref(body), state),
        state.config.line_style,
    )
}

fn enclose_box(block: &Block, style: LineStyle) -> Block {
    let w = block.width;
    let top = box_border(style, w, true);
    let bottom = box_border(style, w, false);
    let side = box_side(style);
    let middle: Vec<String> = block
        .lines
        .iter()
        .map(|l| format!("{side}{l}{side}"))
        .collect();
    let mut lines = Vec::with_capacity(middle.len() + 2);
    lines.push(top);
    lines.extend(middle);
    lines.push(bottom);
    Block {
        lines,
        width: w + 2,
        baseline: block.baseline + 1,
    }
}

fn box_border(style: LineStyle, w: usize, is_top: bool) -> String {
    match style {
        LineStyle::Ascii => "-".repeat(w + 2),
        LineStyle::Unicode => {
            if is_top {
                format!("┌{}┐", "─".repeat(w))
            } else {
                format!("└{}┘", "─".repeat(w))
            }
        }
    }
}

fn box_side(style: LineStyle) -> String {
    match style {
        LineStyle::Ascii => "|".to_string(),
        LineStyle::Unicode => "│".to_string(),
    }
}

fn render_measurement(measurement: &Measurement) -> String {
    format!("{}{}", measurement.number, measurement.unit)
}

fn render_kern(dimension: &Measurement) -> String {
    if dimension.unit != "em" && dimension.unit != "mu" {
        return format!("kern({})", render_measurement(dimension));
    }
    let em = em_value(dimension).unwrap_or_default();
    if em <= 0.0 {
        return String::new();
    }
    let n = (em * 2.0).round();
    let count = if n < 1.0 { 1 } else { n as usize };
    " ".repeat(count)
}

fn render_root(index: &ParseNode, body: &ParseNode, state: &RenderState) -> String {
    render_root_block(index, body, state).render()
}

fn render_root_block(index: &ParseNode, body: &ParseNode, state: &RenderState) -> Block {
    let tight_state = RenderState {
        in_display: false,
        in_tight: true,
        ..state.clone()
    };
    let index_text = render_node_internal(index, &tight_state);
    match unicode_script(&index_text, UnicodeScriptKind::UnicodeSuperscript) {
        Some(prefix) => render_radical_block(&prefix, body, state),
        None => Block::from(&format!("root({index_text},"))
            .beside(&render_operand_block(body, state))
            .beside(&Block::from(")")),
    }
}

fn render_radical(prefix: &str, body: &ParseNode, state: &RenderState) -> String {
    render_radical_block(prefix, body, state).render()
}

fn render_radical_block(prefix: &str, body: &ParseNode, state: &RenderState) -> Block {
    Block::from(&format!("{prefix}√")).beside(&render_operand_block(body, state))
}

fn render_operand(node: &ParseNode, state: &RenderState) -> String {
    render_operand_block(node, state).render()
}

fn render_operand_block(node: &ParseNode, state: &RenderState) -> Block {
    let content = content_block(node, state);
    if is_atomic_expression(node) {
        content
    } else {
        wrap_delims(&content, Some("("), Some(")"))
    }
}

fn render_sup_sub(
    base: &Option<Box<ParseNode>>,
    sup: &Option<Box<ParseNode>>,
    sub: &Option<Box<ParseNode>>,
    state: &RenderState,
) -> String {
    render_sup_sub_block(base, sup, sub, state).render()
}

fn render_sup_sub_block(
    base: &Option<Box<ParseNode>>,
    sup: &Option<Box<ParseNode>>,
    sub: &Option<Box<ParseNode>>,
    state: &RenderState,
) -> Block {
    if let Some(b) = base {
        if operator_uses_limits(b, state) {
            return render_limits_block(b, sup, sub, state);
        }
    }
    let base_block = base
        .as_ref()
        .map_or_else(Block::empty, |b| render_operand_block(b, state));
    let sub_text = render_script(sub, UnicodeScriptKind::UnicodeSubscript, "_", state);
    let sup_text = render_script(sup, UnicodeScriptKind::UnicodeSuperscript, "^", state);
    base_block.beside(&Block::from(&format!("{sub_text}{sup_text}")))
}

fn operator_uses_limits(base: &ParseNode, state: &RenderState) -> bool {
    match base {
        ParseNode::Op {
            limits: true,
            always_handle_sup_sub,
            ..
        } => state.in_display || *always_handle_sup_sub,
        ParseNode::OperatorName {
            always_handle_sup_sub: true,
            ..
        } => true,
        _ => false,
    }
}

fn render_limits_block(
    base: &ParseNode,
    sup: &Option<Box<ParseNode>>,
    sub: &Option<Box<ParseNode>>,
    state: &RenderState,
) -> Block {
    let tight_state = RenderState {
        in_display: false,
        in_tight: true,
        ..state.clone()
    };
    let op_text = render_node_internal(base, &tight_state);
    let sup_text = sup
        .as_ref()
        .map(|s| render_node_internal(s, &tight_state))
        .unwrap_or_default();
    let sub_text = sub
        .as_ref()
        .map(|s| render_node_internal(s, &tight_state))
        .unwrap_or_default();
    let w = display_width(&op_text)
        .max(display_width(&sup_text))
        .max(display_width(&sub_text));
    let mut lines: Vec<String> = Vec::new();
    if !sup_text.is_empty() {
        lines.push(center_text(&sup_text, w));
    }
    lines.push(center_text(&op_text, w));
    let baseline = lines.len() - 1;
    if !sub_text.is_empty() {
        lines.push(center_text(&sub_text, w));
    }
    Block {
        lines,
        width: w,
        baseline,
    }
}

fn render_script(
    node: &Option<Box<ParseNode>>,
    kind: UnicodeScriptKind,
    fallback_prefix: &str,
    state: &RenderState,
) -> String {
    let tight_state = RenderState {
        in_display: false,
        in_tight: true,
        ..state.clone()
    };
    let Some(node) = node else {
        return String::new();
    };
    let text = render_node_internal(node, &tight_state);
    let default = || format!("{fallback_prefix}{}", render_operand(node, &tight_state));
    match (kind, split_prime_prefix(&text)) {
        (UnicodeScriptKind::UnicodeSuperscript, Some((primes, rest))) => {
            match unicode_script(&rest, kind) {
                Some(mapped) => format!("{primes}{mapped}"),
                None => default(),
            }
        }
        _ => unicode_script(&text, kind).unwrap_or_else(default),
    }
}

fn split_prime_prefix(text: &str) -> Option<(String, String)> {
    let mut count = 0;
    for c in text.chars() {
        if c == '′' {
            count += 1;
        } else {
            break;
        }
    }
    if count == 0 {
        return None;
    }
    let primes = "'".repeat(count);
    let rest: String = text.chars().skip(count).collect();
    Some((primes, rest))
}

fn render_function_block(name: &str, body: &Block) -> Block {
    Block::from(&format!("{name}("))
        .beside(body)
        .beside(&Block::from(")"))
}

fn render_font_block(font: &str, body: &ParseNode, state: &RenderState) -> Block {
    if font == "mathrm" {
        return content_block(body, state);
    }
    match font_char_sequence(font, body) {
        Some(rendered) => Block::from(&rendered),
        None => render_function_block(font, &content_block(body, state)),
    }
}

fn render_operator_content_block(content: &OperatorContent, state: &RenderState) -> Block {
    match content {
        OperatorContent::SymbolOperator(text) => Block::from(&resolve_symbol(text)),
        OperatorContent::NamedOperator(text) => Block::from(&command_name(text)),
        OperatorContent::BodyOperator(body) => content_body_block(body, state),
    }
}

pub(crate) fn content_block(node: &ParseNode, state: &RenderState) -> Block {
    match node {
        ParseNode::Unsupported { fallback, .. } => content_block(fallback, state),
        ParseNode::Styling { body, style, .. } => content_body_block(
            body,
            &RenderState {
                style: *style,
                in_display: state.in_display || *style == StyleLevel::DisplayStyle,
                ..state.clone()
            },
        ),
        ParseNode::Text { body, .. } => content_body_block(
            body,
            &RenderState {
                in_tight: true,
                ..state.clone()
            },
        ),
        ParseNode::OrdGroup { body, .. }
        | ParseNode::MClass { body, .. }
        | ParseNode::HBox { body, .. }
        | ParseNode::Sizing { body, .. }
        | ParseNode::Color { body, .. }
        | ParseNode::Href { body, .. }
        | ParseNode::Html { body, .. } => content_body_block(body, state),
        ParseNode::Smash { body, .. }
        | ParseNode::VCenter { body, .. }
        | ParseNode::RaiseBox { body, .. }
        | ParseNode::ReflectBox { body, .. }
        | ParseNode::Lap { body, .. } => content_block(body, state),
        ParseNode::CdParent { fragment, .. } => content_block(fragment, state),
        ParseNode::GenFrac {
            numer,
            denom,
            has_bar_line: false,
            left_delim: None,
            right_delim: None,
            ..
        } if state.in_display => frac_block(numer, denom, state, false),
        ParseNode::GenFrac {
            numer,
            denom,
            has_bar_line: false,
            left_delim,
            right_delim,
            ..
        } if state.in_display => barless_delimited_block(
            numer,
            denom,
            left_delim.as_deref(),
            right_delim.as_deref(),
            state,
        ),
        ParseNode::GenFrac {
            numer,
            denom,
            left_delim: None,
            right_delim: None,
            ..
        } if state.in_display => frac_block(numer, denom, state, true),
        ParseNode::GenFrac {
            numer,
            denom,
            has_bar_line,
            left_delim,
            right_delim,
            ..
        } if state.in_display => wrap_delims(
            &frac_block(numer, denom, state, *has_bar_line),
            left_delim.as_deref(),
            right_delim.as_deref(),
        ),
        ParseNode::LeftRight {
            body, left, right, ..
        } if state.in_display => render_leftright_display_block(body, left, right, state),
        ParseNode::Array {
            body,
            column_separation_type: Some(ColumnSeparationType::CdSeparation),
            ..
        } => render_cd_block(body, state),
        ParseNode::Array {
            body,
            columns,
            hlines_before_row,
            column_separation_type,
            ..
        } if state.in_display => cell_block(
            body,
            columns.as_deref(),
            hlines_before_row,
            *column_separation_type,
            state,
        ),
        ParseNode::Array { .. } => Block::from(&render_node_internal(node, state)),
        ParseNode::Enclose { label, body, .. } if label == "\\fbox" => {
            render_enclose_block(body, state)
        }
        ParseNode::Sqrt {
            body, index: None, ..
        } => render_radical_block("", body, state),
        ParseNode::Sqrt {
            body,
            index: Some(index),
            ..
        } => render_root_block(index, body, state),
        ParseNode::SupSub { base, sup, sub, .. } => render_sup_sub_block(base, sup, sub, state),
        ParseNode::Overline { body, .. } => {
            render_function_block("overline", &content_block(body, state))
        }
        ParseNode::Underline { body, .. } => {
            render_function_block("underline", &content_block(body, state))
        }
        ParseNode::VPhantom { body, .. } => {
            render_function_block("vphantom", &content_block(body, state))
        }
        ParseNode::Phantom { body, .. } => {
            render_function_block("phantom", &content_body_block(body, state))
        }
        ParseNode::Pmb { body, .. } => {
            render_function_block("bold", &content_body_block(body, state))
        }
        ParseNode::Enclose { label, body, .. } => {
            render_function_block(&command_name(label), &content_block(body, state))
        }
        ParseNode::HorizBrace { label, base, .. } => {
            render_function_block(&command_name(label), &content_block(base, state))
        }
        ParseNode::XArrow {
            label,
            body,
            below: None,
            ..
        } => render_function_block(&command_name(label), &content_block(body, state)),
        ParseNode::XArrow {
            label,
            body,
            below: Some(below),
            ..
        } => render_function_block(
            &command_name(label),
            &content_block(body, state)
                .beside(&Block::from(","))
                .beside(&content_block(below, state)),
        ),
        ParseNode::AccentUnder { label, base, .. } => {
            Block::from(&format!("underaccent({},", command_name(label)))
                .beside(&content_block(base, state))
                .beside(&Block::from(")"))
        }
        ParseNode::Accent { label, base, .. } => render_accent_block(label, base, state),
        ParseNode::Font { font, body, .. } => render_font_block(font, body, state),
        ParseNode::Op { content, .. } => render_operator_content_block(content, state),
        ParseNode::OperatorName { body, .. } => content_body_block(body, state),
        ParseNode::MathChoice {
            display,
            text,
            script,
            scriptscript,
            ..
        } => content_body_block(
            &math_choice_variant(display, text, script, scriptscript, state.style),
            state,
        ),
        ParseNode::Tag { body, tag, .. } => content_body_block(body, state)
            .beside(&Block::from("\t"))
            .beside(&content_body_block(tag, state)),
        ParseNode::HtmlMathML { mathml, .. } => content_body_block(mathml, state),
        ParseNode::Internal { .. } | ParseNode::EnvironmentEnd { .. } => Block::empty(),
        _ => Block::from(&render_node_internal(node, state)),
    }
}

fn content_body_block(body: &[ParseNode], state: &RenderState) -> Block {
    match body {
        [] => Block::empty(),
        [single] => content_block(single, state),
        _ => render_internal_block(body, state),
    }
}

fn unicode_script(text: &str, kind: UnicodeScriptKind) -> Option<String> {
    let mut acc: Option<String> = Some(String::new());
    for character in text.chars() {
        let ch = character.to_string();
        acc = match acc {
            None => None,
            Some(s) => unicode_script_character(kind, &ch).map(|script| format!("{s}{script}")),
        };
    }
    acc
}

fn render_accent(label: &str, base: &ParseNode, state: &RenderState) -> String {
    render_accent_block(label, base, state).render()
}

fn render_accent_block(label: &str, base: &ParseNode, state: &RenderState) -> Block {
    let mark = match label {
        "\\acute" => Some("\u{301}"),
        "\\grave" => Some("\u{300}"),
        "\\ddot" => Some("\u{308}"),
        "\\tilde" => Some("\u{303}"),
        "\\bar" => Some("\u{304}"),
        "\\breve" => Some("\u{306}"),
        "\\check" => Some("\u{30C}"),
        "\\hat" => Some("\u{302}"),
        "\\dot" => Some("\u{307}"),
        "\\mathring" => Some("\u{30A}"),
        "\\vec" => Some("\u{20D7}"),
        _ => None,
    };
    let rendered_base = render_node_internal(base, state);
    match mark {
        Some(mark) if is_single_character_output(&rendered_base) => {
            Block::from(&format!("{rendered_base}{mark}"))
        }
        _ => render_function_block(&command_name(label), &content_block(base, state)),
    }
}

fn is_single_character_output(s: &str) -> bool {
    s.chars().count() == 1
}
