use super::*;
struct Row {
    cells: Vec<Node>,
    height: f64,
    depth: f64,
    position: f64,
}
fn gap(width: f64) -> Node {
    let mut n = span(&["arraycolsep"], vec![], None);
    n.set_style("width", make_em(width));
    n
}
fn place_lines(lines: &[bool], total: &mut f64, horizontal: &mut Vec<(f64, bool)>) {
    for (i, dashed) in lines.iter().enumerate() {
        if i > 0 {
            *total += 0.25;
        }
        horizontal.push((*total, *dashed));
    }
}
pub(super) fn array(node: &ParseNode, o: &Options) -> Node {
    let ParseNode::Array {
        body: input,
        add_jot,
        array_stretch,
        columns,
        row_gaps,
        hskip_before_and_after: hskip,
        hlines_before_row: hlines,
        column_separation_type: separation,
        tags,
        auto_tags,
        ..
    } = node
    else {
        return Node::error("Expected array");
    };
    let mut rows = vec![];
    let mut horizontal = vec![];
    let rule = o.metric("arrayRuleWidth").max(o.min_rule_thickness);
    let point = 1.0 / o.metric("ptPerEm");
    let col_sep = if *separation == Some(ColumnSeparationType::SmallSeparation) {
        0.2778 * (o.with_style(4).size_multiplier / o.size_multiplier)
    } else {
        5.0 * point
    };
    let baseline = if *separation == Some(ColumnSeparationType::CdSeparation) {
        calculate_size(
            &Measurement {
                number: 3.0,
                unit: "ex".into(),
            },
            o,
        )
    } else {
        12.0 * point
    };
    let skip = array_stretch * baseline;
    let sh = 0.7 * skip;
    let sd = 0.3 * skip;
    let mut total = 0.0;
    let mut count = 0;
    place_lines(
        hlines.first().map_or(&[], Vec::as_slice),
        &mut total,
        &mut horizontal,
    );
    for (i, row) in input.iter().enumerate() {
        let (mut height, mut depth) = (sh, sd);
        count = count.max(row.len());
        let cells = row
            .iter()
            .map(|cell| {
                let e = build_group(cell, o);
                height = height.max(e.height);
                depth = depth.max(e.depth);
                e
            })
            .collect();
        let mut gap = row_gaps
            .get(i)
            .and_then(Option::as_ref)
            .map_or(0.0, |s| calculate_size(s, o));
        if gap > 0.0 {
            depth = depth.max(gap + sd);
            gap = 0.0;
        }
        if *add_jot && i + 1 < input.len() {
            depth += 3.0 * point;
        }
        total += height;
        rows.push(Row {
            cells,
            height,
            depth,
            position: total,
        });
        total += depth + gap;
        place_lines(
            hlines.get(i + 1).map_or(&[], Vec::as_slice),
            &mut total,
            &mut horizontal,
        );
    }
    let offset = total / 2.0 + o.metric("axisHeight");
    let descriptions = columns.as_deref().unwrap_or(&[]);
    let mut output = vec![];
    let mut tag_spans = vec![];
    let has_tags = tags.as_ref().is_some_and(|t| t.iter().any(Option::is_some))
        || auto_tags.as_ref().is_some_and(|t| t.contains(&true));
    if has_tags {
        for (i, row) in rows.iter().enumerate() {
            let automatic = auto_tags
                .as_ref()
                .and_then(|t| t.get(i))
                .copied()
                .unwrap_or(false);
            let manual = tags
                .as_ref()
                .and_then(|t| t.get(i))
                .and_then(Option::as_ref);
            let mut tag = manual.map_or_else(
                || span(if automatic { &["eqn-num"] } else { &[] }, vec![], Some(o)),
                |b| span(&[], build_expression(b, o), Some(o)),
            );
            tag.height = row.height;
            tag.depth = row.depth;
            tag_spans.push(shifted(tag, row.position - offset));
        }
    }
    let (mut ci, mut di) = (0, 0);
    while ci < count || di < descriptions.len() {
        let mut first = true;
        while let Some(ArrayColumn::SeparatorColumn { separator }) = descriptions.get(di) {
            if !first {
                output.push(gap(o.metric("doubleRuleSep")));
            }
            let mut line = span(&["vertical-separator"], vec![], Some(o));
            line.set_style("height", make_em(total));
            line.set_style("borderRightWidth", make_em(rule));
            line.set_style(
                "borderRightStyle",
                if separator == ":" { "dashed" } else { "solid" },
            );
            line.set_style("margin", format!("0 {}", make_em(-rule / 2.0)));
            let shift = total - offset;
            if shift != 0.0 {
                line.set_style("verticalAlign", make_em(-shift));
            }
            output.push(line);
            di += 1;
            first = false;
        }
        if ci < count {
            let (align, pre, post) = match descriptions.get(di) {
                Some(ArrayColumn::AlignColumn {
                    alignment,
                    pre_gap,
                    post_gap,
                }) => (
                    alignment.as_str(),
                    pre_gap.unwrap_or(col_sep),
                    post_gap.unwrap_or(col_sep),
                ),
                _ => ("c", col_sep, col_sep),
            };
            if (ci > 0 || *hskip) && pre != 0.0 {
                output.push(gap(pre));
            }
            let mut cells = vec![];
            for row in &rows {
                if let Some(c) = row.cells.get(ci) {
                    let mut cell = c.clone();
                    cell.height = row.height;
                    cell.depth = row.depth;
                    cells.push(shifted(cell, row.position - offset));
                }
            }
            output.push(span(
                &[&format!("col-align-{align}")],
                vec![vlist("individualShift", 0.0, cells)],
                None,
            ));
            if (ci + 1 < count || *hskip) && post != 0.0 {
                output.push(gap(post));
            }
        }
        ci += 1;
        di += 1;
    }
    let mut table = span(&["mtable"], output, None);
    if !horizontal.is_empty() {
        let line = line_span("katex-hline", o, Some(rule));
        let dash = line_span("katex-hdashline", o, Some(rule));
        let mut elements = vec![elem(table)];
        for (position, dashed) in horizontal.into_iter().rev() {
            elements.push(shifted(
                if dashed { dash.clone() } else { line.clone() },
                position - offset,
            ));
        }
        table = vlist("individualShift", 0.0, elements);
    }
    if tag_spans.is_empty() {
        span(&["mord"], vec![table], Some(o))
    } else {
        fragment(vec![
            table,
            span(
                &["katex-tag"],
                vec![vlist("individualShift", 0.0, tag_spans)],
                Some(o),
            ),
        ])
    }
}
