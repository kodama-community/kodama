//! TeX math spacing rules. Based on KaTeX spacingData.ts and TeXbook Ch.18.

use crate::ast::AtomFamily;

/// Concrete space representations for a rendering backend.
/// Different backends (unicode text, HTML/CSS, LaTeX) provide their own spec.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpacingSpec {
    pub thick: String,
    pub medium: String,
    pub thin: String,
    pub operator: String,
}

/// Converts AtomFamily to the corresponding spacing type string.
pub fn atom_family_name(family: AtomFamily) -> String {
    match family {
        AtomFamily::Mord => "mord",
        AtomFamily::Mop => "mop",
        AtomFamily::Mbin => "mbin",
        AtomFamily::Mrel => "mrel",
        AtomFamily::Mopen => "mopen",
        AtomFamily::Mclose => "mclose",
        AtomFamily::Mpunct => "mpunct",
        AtomFamily::Minner => "minner",
    }
    .to_string()
}

/// Normal (display/text) spacing table between atom types.
fn normal_spacing<'a>(left: &str, right: &str, spec: &'a SpacingSpec) -> Option<&'a str> {
    match (left, right) {
        ("mord", "mop" | "mbig") => Some(&spec.thin),
        ("mord", "mbin") => Some(&spec.medium),
        ("mord", "mrel") => Some(&spec.thick),
        ("mord", "minner") => Some(&spec.thin),
        ("mop", "mord") => Some(&spec.operator),
        ("mop", "mop" | "mbig") => Some(&spec.operator),
        ("mop", "mrel") => Some(&spec.thick),
        ("mop", "minner") => Some(&spec.thin),
        ("mbig", "mord" | "mop" | "mopen" | "minner") => Some(&spec.medium),
        ("mbig", "mbin") => Some(&spec.medium),
        ("mbig", "mrel") => Some(&spec.thick),
        ("mbig", "mclose" | "mpunct") => Some(&spec.thin),
        ("mbin", "mord") => Some(&spec.medium),
        ("mbin", "mop" | "mbig") => Some(&spec.medium),
        ("mbin", "mopen") => Some(&spec.medium),
        ("mbin", "minner") => Some(&spec.medium),
        ("mrel", "mord") => Some(&spec.thick),
        ("mrel", "mop" | "mbig") => Some(&spec.thick),
        ("mrel", "mopen") => Some(&spec.thick),
        ("mrel", "minner") => Some(&spec.thick),
        ("mclose", "mop" | "mbig") => Some(&spec.thin),
        ("mclose", "mbin") => Some(&spec.medium),
        ("mclose", "mrel") => Some(&spec.thick),
        ("mclose", "minner") => Some(&spec.thin),
        ("mpunct", "mord") => Some(&spec.thin),
        ("mpunct", "mop" | "mbig") => Some(&spec.thin),
        ("mpunct", "mrel") => Some(&spec.thick),
        ("mpunct", "mopen") => Some(&spec.thin),
        ("mpunct", "mclose") => Some(&spec.thin),
        ("mpunct", "mpunct") => Some(&spec.thin),
        ("mpunct", "minner") => Some(&spec.thin),
        ("minner", "mord") => Some(&spec.thin),
        ("minner", "mop" | "mbig") => Some(&spec.thin),
        ("minner", "mbin") => Some(&spec.medium),
        ("minner", "mrel") => Some(&spec.thick),
        ("minner", "mopen") => Some(&spec.thin),
        ("minner", "mpunct") => Some(&spec.thin),
        ("minner", "minner") => Some(&spec.thin),
        _ => None,
    }
}

/// Tight (script/scriptscript) spacing table.
fn tight_spacing<'a>(left: &str, right: &str, spec: &'a SpacingSpec) -> Option<&'a str> {
    match (left, right) {
        ("mord", "mop") => Some(&spec.thin),
        ("mop", "mord") => Some(&spec.thin),
        ("mop", "mop") => Some(&spec.thin),
        ("mbig", "mord" | "mop" | "minner") => Some(&spec.thin),
        ("mclose", "mop") => Some(&spec.thin),
        ("minner", "mop") => Some(&spec.thin),
        _ => None,
    }
}

/// Look up spacing between two atom types.
pub fn math_spacing(
    left_type: &str,
    right_type: &str,
    tight: bool,
    spec: &SpacingSpec,
) -> Option<String> {
    if tight {
        tight_spacing(left_type, right_type, spec)
    } else {
        normal_spacing(left_type, right_type, spec)
    }
    .map(ToString::to_string)
}

/// A renderable atom with its math class and baseline info.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpacableItem {
    pub atom_type: Option<String>,
    pub text: String,
    pub baseline: usize,
}

/// True when a type to the left of a binary atom cancels its binary spacing.
/// TeXbook Rules 5-6.
fn cancels_left_of_bin(ty: &str) -> bool {
    matches!(
        ty,
        "leftmost" | "mbin" | "mopen" | "mrel" | "mop" | "mpunct" | "mbig"
    )
}

/// True when a type to the right of a binary atom cancels its binary spacing.
fn cancels_right_of_bin(ty: &str) -> bool {
    matches!(ty, "rightmost" | "mrel" | "mclose" | "mpunct")
}

/// Apply bin cancellation: Mbin atoms become Mord in certain contexts.
pub fn cancel_bin_atoms(items: Vec<SpacableItem>) -> Vec<SpacableItem> {
    let n = items.len();
    if n == 0 {
        return items;
    }
    let mut items = items;

    if items[0].atom_type.as_deref() == Some("mbin") {
        items[0].atom_type = Some("mord".to_string());
    }

    for i in 1..n {
        let prev_ty = items[i - 1].atom_type.clone();
        let curr_ty = items[i].atom_type.clone();
        match (prev_ty.as_deref(), curr_ty.as_deref()) {
            (Some("mbin"), Some(right)) => {
                if cancels_right_of_bin(right) {
                    items[i - 1].atom_type = Some("mord".to_string());
                }
            }
            (Some(left), Some("mbin")) if cancels_left_of_bin(left) => {
                items[i].atom_type = Some("mord".to_string());
            }
            _ => (),
        }
    }

    if n >= 1 && items[n - 1].atom_type.as_deref() == Some("mbin") {
        items[n - 1].atom_type = Some("mord".to_string());
    }

    items
}

/// Join spacable items into a string, inserting spacing between adjacent
/// non-space atoms according to the given `SpacingSpec`.
pub fn join_with_spacing(items: &[SpacableItem], tight: bool, spec: &SpacingSpec) -> String {
    let n = items.len();
    if n == 0 {
        return String::new();
    }
    if n == 1 {
        return items[0].text.clone();
    }

    let mut result = items[0].text.clone();
    for i in 1..n {
        let prev = &items[i - 1];
        let curr = &items[i];
        match (&prev.atom_type, &curr.atom_type) {
            (Some(left), Some(right)) => {
                if let Some(space) = math_spacing(left, right, tight, spec) {
                    result.push_str(&space);
                }
                result.push_str(&curr.text);
            }
            _ => result.push_str(&curr.text),
        }
    }
    result
}
