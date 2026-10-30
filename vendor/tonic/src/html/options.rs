use super::*;

/// Configuration for KaTeX-compatible HTML output (`output: "html"`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderConfig {
    pub display_mode: bool,
    pub leqno: bool,
    pub fleqn: bool,
    pub max_size: f64,
    pub min_rule_thickness: f64,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            display_mode: false,
            leqno: false,
            fleqn: false,
            max_size: f64::INFINITY,
            min_rule_thickness: 0.0,
        }
    }
}

impl RenderConfig {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Clone, Debug)]
pub(super) struct Options {
    pub style: usize,
    pub size: usize,
    pub text_size: usize,
    pub size_multiplier: f64,
    pub font: String,
    pub font_family: String,
    pub font_weight: String,
    pub font_shape: String,
    pub color: Option<String>,
    pub phantom: bool,
    pub max_size: f64,
    pub min_rule_thickness: f64,
}

const MULTIPLIERS: [f64; 11] = [0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.2, 1.44, 1.728, 2.074, 2.488];
const SIZE_STYLE_MAP: [[usize; 3]; 11] = [
    [1, 1, 1],
    [2, 1, 1],
    [3, 1, 1],
    [4, 2, 1],
    [5, 2, 1],
    [6, 3, 1],
    [7, 4, 2],
    [8, 6, 3],
    [9, 7, 6],
    [10, 8, 7],
    [11, 10, 9],
];

fn size_at_style(size: usize, style: usize) -> usize {
    if style < 4 {
        size
    } else {
        SIZE_STYLE_MAP[size - 1][style / 2 - 1]
    }
}

impl Options {
    pub fn new(config: RenderConfig) -> Self {
        Self {
            style: if config.display_mode { 0 } else { 2 },
            size: 6,
            text_size: 6,
            size_multiplier: 1.0,
            font: String::new(),
            font_family: String::new(),
            font_weight: String::new(),
            font_shape: String::new(),
            color: None,
            phantom: false,
            max_size: config.max_size,
            min_rule_thickness: config.min_rule_thickness,
        }
    }
    pub fn with_style(&self, style: usize) -> Self {
        if style == self.style {
            return self.clone();
        }
        let size = size_at_style(self.text_size, style);
        Self {
            style,
            size,
            size_multiplier: MULTIPLIERS[size - 1],
            ..self.clone()
        }
    }
    pub fn with_size(&self, size: usize) -> Self {
        if self.size == size && self.text_size == size {
            return self.clone();
        }
        Self {
            style: style_text(self.style),
            size,
            text_size: size,
            size_multiplier: MULTIPLIERS[size - 1],
            ..self.clone()
        }
    }
    pub fn base_style(&self, style: usize) -> Self {
        let size = size_at_style(6, style);
        Self {
            style,
            size,
            size_multiplier: MULTIPLIERS[size - 1],
            ..self.clone()
        }
    }
    pub fn base_sizing(&self) -> Self {
        let size = if self.style >= 6 {
            1
        } else if self.style >= 4 {
            3
        } else {
            6
        };
        Self {
            style: style_text(self.style),
            size,
            size_multiplier: MULTIPLIERS[size - 1],
            ..self.clone()
        }
    }
    pub fn sizing_classes(&self, old: &Self) -> Vec<String> {
        if self.size == old.size {
            vec![]
        } else {
            vec![
                "katex-sizing".into(),
                format!("reset-size{}", old.size),
                format!("size{}", self.size),
            ]
        }
    }
    pub fn base_sizing_classes(&self) -> Vec<String> {
        if self.size == 6 {
            vec![]
        } else {
            vec![
                "katex-sizing".into(),
                format!("reset-size{}", self.size),
                "size6".into(),
            ]
        }
    }
    pub fn get_color(&self) -> Option<&str> {
        if self.phantom {
            Some("transparent")
        } else {
            self.color.as_deref()
        }
    }
    pub fn metric(&self, name: &str) -> f64 {
        if name == "cssEmPerMu" {
            self.metric("quad") / 18.0
        } else {
            data::global_metric(name)[if self.size >= 5 {
                0
            } else if self.size >= 3 {
                1
            } else {
                2
            }]
        }
    }
}
pub(super) fn style_sup(s: usize) -> usize {
    [4, 5, 4, 5, 6, 7, 6, 7][s]
}
pub(super) fn style_sub(s: usize) -> usize {
    [5, 5, 5, 5, 7, 7, 7, 7][s]
}
pub(super) fn style_frac_num(s: usize) -> usize {
    [2, 3, 4, 5, 6, 7, 6, 7][s]
}
pub(super) fn style_frac_den(s: usize) -> usize {
    [3, 3, 5, 5, 7, 7, 7, 7][s]
}
pub(super) fn style_cramp(s: usize) -> usize {
    [1, 1, 3, 3, 5, 5, 7, 7][s]
}
pub(super) fn style_text(s: usize) -> usize {
    [0, 1, 2, 3, 2, 3, 2, 3][s]
}
pub(super) fn style_id(s: StyleLevel) -> usize {
    match s {
        StyleLevel::DisplayStyle => 0,
        StyleLevel::TextStyle => 2,
        StyleLevel::ScriptStyle => 4,
        StyleLevel::ScriptScriptStyle => 6,
    }
}

pub(super) fn calculate_size(value: &Measurement, o: &Options) -> f64 {
    let absolute = match value.unit.as_str() {
        "pt" => Some(1.0),
        "mm" => Some(7227.0 / 2540.0),
        "cm" => Some(7227.0 / 254.0),
        "in" => Some(72.27),
        "bp" | "px" => Some(803.0 / 800.0),
        "pc" => Some(12.0),
        "dd" => Some(1238.0 / 1157.0),
        "cc" => Some(14856.0 / 1157.0),
        "nd" => Some(685.0 / 642.0),
        "nc" => Some(1370.0 / 107.0),
        "sp" => Some(1.0 / 65536.0),
        _ => None,
    };
    let scale = if let Some(scale) = absolute {
        scale / o.metric("ptPerEm") / o.size_multiplier
    } else if value.unit == "mu" {
        o.metric("cssEmPerMu")
    } else {
        let unit_options = if o.style >= 4 {
            o.with_style(style_text(o.style))
        } else {
            o.clone()
        };
        unit_options.metric(if value.unit == "ex" {
            "xHeight"
        } else {
            "quad"
        }) * unit_options.size_multiplier
            / o.size_multiplier
    };
    (value.number * scale).min(o.max_size)
}

/// Match JavaScript's +n.toFixed(4), including binary halfway rounding.
pub(super) fn make_em(value: f64) -> String {
    if !value.is_finite() || value.abs() >= 1e14 {
        return format!("{}em", js_number(value));
    }
    let bits = value.abs().to_bits();
    let exponent = ((bits >> 52) & 0x7ff) as i32;
    let significand = (bits & 0xfffffffffffff) | if exponent == 0 { 0 } else { 1 << 52 };
    let product = u128::from(significand) * 625;
    let shift = if exponent == 0 { 1070 } else { 1071 - exponent };
    let scaled = if shift >= 128 {
        0
    } else if shift <= 0 {
        product << -shift
    } else {
        (product >> shift)
            + u128::from((product & ((1u128 << shift) - 1)) >= (1u128 << (shift - 1)))
    };
    if scaled == 0 {
        return "0em".into();
    }
    let sign = if value < 0.0 { "-" } else { "" };
    let integer = scaled / 10000;
    let fraction = scaled % 10000;
    if fraction == 0 {
        format!("{sign}{integer}em")
    } else {
        format!(
            "{sign}{integer}.{}em",
            format!("{fraction:04}").trim_end_matches('0')
        )
    }
}
pub(super) fn js_number(n: f64) -> String {
    if n == 0.0 {
        "0".into()
    } else if n == f64::INFINITY {
        "Infinity".into()
    } else if n == f64::NEG_INFINITY {
        "-Infinity".into()
    } else {
        n.to_string()
    }
}
