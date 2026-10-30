use super::*;

#[derive(Clone, Copy, Default)]
pub(super) struct CharacterMetrics {
    pub depth: f64,
    pub height: f64,
    pub italic: f64,
    pub skew: f64,
    pub width: f64,
}
pub(super) fn unicode_script(c: u32) -> Option<&'static str> {
    match c {
        0x0100..=0x024f | 0x0300..=0x036f => Some("latin"),
        0x0400..=0x04ff => Some("cyrillic"),
        0x0530..=0x058f => Some("armenian"),
        0x0900..=0x109f => Some("brahmic"),
        0x10a0..=0x10ff => Some("georgian"),
        0x3000..=0x30ff | 0x4e00..=0x9faf | 0xff00..=0xff60 => Some("cjk"),
        0xac00..=0xd7af => Some("hangul"),
        _ => None,
    }
}
pub(super) fn character_metrics(
    character: &str,
    font: &str,
    mode: Mode,
) -> Option<CharacterMetrics> {
    let code = character.chars().next()? as u32;
    let values = data::character_metric(font, code)
        .or_else(|| data::character_metric(font, data::extra_character(code)))
        .or_else(|| {
            if mode == Mode::Text && unicode_script(code).is_some() {
                data::character_metric(font, 77)
            } else {
                None
            }
        })?;
    Some(CharacterMetrics {
        depth: values[0],
        height: values[1],
        italic: values[2],
        skew: values[3],
        width: values[4],
    })
}
pub(super) fn symbol_info(value: &str, mode: Mode) -> Option<(&'static str, &'static str)> {
    data::symbol(if mode == Mode::Math { "math" } else { "text" }, value)
}
pub(super) fn symbol_value(value: &str, mode: Mode) -> &str {
    symbol_info(value, mode).map_or(value, |e| e.1)
}
pub(super) fn lookup_metrics(value: &str, font: &str, mode: Mode) -> Option<CharacterMetrics> {
    character_metrics(symbol_value(value, mode), font, mode)
}
pub(super) fn make_symbol(
    value: &str,
    font: &str,
    mode: Mode,
    o: &Options,
    names: Vec<String>,
) -> Node {
    let value = symbol_value(value, mode);
    let mut n = Node::new("symbol", names, vec![]);
    if let Some(m) = character_metrics(value, font, mode) {
        n.height = m.height;
        n.depth = m.depth;
        n.italic = if mode == Mode::Text || o.font == "mathit" {
            0.0
        } else {
            m.italic
        };
        n.skew = m.skew;
        n.width = m.width;
    }
    if let Some(script) = value.chars().next().and_then(|c| unicode_script(c as u32)) {
        n.classes.push(format!("{script}_fallback"));
    }
    n.text = match value {
        "î" => "ı̂",
        "ï" => "ı̈",
        "í" => "ı́",
        "ì" => "ı̀",
        _ => value,
    }
    .into();
    n.max_font_size = o.size_multiplier;
    if o.style >= 4 {
        n.classes.push("mtight".into());
    }
    if let Some(color) = o.get_color() {
        n.set_style("color", color);
    }
    n
}
pub(super) fn math_symbol(value: &str, mode: Mode, o: &Options, mut names: Vec<String>) -> Node {
    let font = if o.font == "boldsymbol" && lookup_metrics(value, "Main-Bold", mode).is_some() {
        names.push("mathbf".into());
        "Main-Bold"
    } else if value == "\\" || symbol_info(value, mode).is_none_or(|s| s.0 == "main") {
        "Main-Regular"
    } else {
        names.push("amsrm".into());
        "AMS-Regular"
    };
    make_symbol(value, font, mode, o, names)
}
fn text_font_name(family: &str, weight: &str, shape: &str) -> String {
    let base = match family {
        "amsrm" => "AMS",
        "textrm" | "" => "Main",
        "textsf" => "SansSerif",
        "texttt" => "Typewriter",
        _ => family,
    };
    let suffix = if weight == "textbf" && shape == "textit" {
        "BoldItalic"
    } else if weight == "textbf" {
        "Bold"
    } else if shape == "textit" {
        "Italic"
    } else {
        "Regular"
    };
    format!("{base}-{suffix}")
}
fn math_font_name(font: &str) -> &str {
    match font {
        "mathbf" => "Main-Bold",
        "mathrm" => "Main-Regular",
        "textit" | "mathit" => "Main-Italic",
        "mathnormal" => "Math-Italic",
        "mathsfit" => "SansSerif-Italic",
        "mathbb" => "AMS-Regular",
        "mathcal" => "Caligraphic-Regular",
        "mathfrak" => "Fraktur-Regular",
        "mathscr" => "Script-Regular",
        "mathsf" => "SansSerif-Regular",
        "mathtt" => "Typewriter-Regular",
        _ => "Main-Regular",
    }
}
fn wide_font(text: &str, mode: Mode) -> Option<(&'static str, &'static str)> {
    let code = text.chars().next()? as u32;
    let index = match code {
        0x1d400..=0x1d6a3 => ((code - 0x1d400) / 52) as i32,
        0x1d7ce..=0x1d7ff => [0, -1, 8, 9, 12][((code - 0x1d7ce) / 10) as usize],
        0x1d6a5 | 0x1d6a6 => 0,
        _ => -1,
    };
    let (font, math, text) = match index {
        0 => ("Main-Bold", "mathbf", "textbf"),
        1 => ("Math-Italic", "mathnormal", "textit"),
        2 => ("Main-BoldItalic", "boldsymbol", "boldsymbol"),
        3 if code < 0x1d4b6 => ("Script-Regular", "mathscr", "textscr"),
        5 => ("Fraktur-Regular", "mathfrak", "textfrak"),
        6 => ("AMS-Regular", "mathbb", "textbb"),
        7 => ("Fraktur-Regular", "mathboldfrak", "textboldfrak"),
        8 => ("SansSerif-Regular", "mathsf", "textsf"),
        9 => ("SansSerif-Bold", "mathboldsf", "textboldsf"),
        10 => ("SansSerif-Italic", "mathitsf", "textitsf"),
        12 => ("Typewriter-Regular", "mathtt", "texttt"),
        _ => return None,
    };
    Some((font, if mode == Mode::Math { math } else { text }))
}
pub(super) fn make_ord(group: &ParseNode, o: &Options) -> Node {
    let (mode, text, mathord) = match group {
        ParseNode::MathOrd { mode, text, .. } => (*mode, text.as_str(), true),
        ParseNode::TextOrd { mode, text, .. } | ParseNode::Spacing { mode, text, .. } => {
            (*mode, text.as_str(), false)
        }
        _ => return span(&[], vec![], None),
    };
    if let Some((font, class)) = wide_font(text, mode) {
        return make_symbol(text, font, mode, o, classes(&["mord", class]));
    }
    let use_font = mode == Mode::Math || !o.font.is_empty();
    let font_or_family = if use_font { &o.font } else { &o.font_family };
    if !font_or_family.is_empty() {
        let (font, names) = if font_or_family == "boldsymbol" {
            if mathord && lookup_metrics(text, "Math-BoldItalic", mode).is_some() {
                ("Math-BoldItalic".into(), classes(&["boldsymbol"]))
            } else {
                ("Main-Bold".into(), classes(&["mathbf"]))
            }
        } else if use_font {
            (math_font_name(&o.font).into(), vec![o.font.clone()])
        } else {
            (
                text_font_name(&o.font_family, &o.font_weight, &o.font_shape),
                vec![
                    o.font_family.clone(),
                    o.font_weight.clone(),
                    o.font_shape.clone(),
                ],
            )
        };
        let mut all = classes(&["mord"]);
        all.extend(names);
        if lookup_metrics(text, &font, mode).is_some() {
            return make_symbol(text, &font, mode, o, all);
        }
        if matches!(text, "--" | "---" | "``" | "''") && font.starts_with("Typewriter") {
            return fragment(
                text.chars()
                    .map(|c| make_symbol(&c.to_string(), &font, mode, o, all.clone()))
                    .collect(),
            );
        }
    }
    if mathord {
        make_symbol(
            text,
            "Math-Italic",
            mode,
            o,
            classes(&["mord", "mathnormal"]),
        )
    } else {
        let ams = symbol_info(text, mode).is_some_and(|s| s.0 == "ams");
        let mut names = classes(&["mord"]);
        if ams {
            names.push("amsrm".into());
        }
        names.extend([o.font_weight.clone(), o.font_shape.clone()]);
        make_symbol(
            text,
            &text_font_name(
                if ams { "amsrm" } else { "textrm" },
                &o.font_weight,
                &o.font_shape,
            ),
            mode,
            o,
            names,
        )
    }
}
