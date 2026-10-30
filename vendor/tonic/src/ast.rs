use crate::source_location::SourceLocation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The parsing mode: math or text.
pub enum Mode {
    Math,
    Text,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The math atom class of a symbol, used for spacing decisions.
pub enum AtomFamily {
    Mord,
    Mop,
    Mbin,
    Mrel,
    Mopen,
    Mclose,
    Mpunct,
    Minner,
}

#[derive(Debug, Clone, PartialEq)]
/// A dimension with a numeric value and a unit (e.g. `em`, `pt`).
pub struct Measurement {
    pub number: f64,
    pub unit: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The typesetting style level, as selected by \\displaystyle and friends.
pub enum StyleLevel {
    DisplayStyle,
    TextStyle,
    ScriptStyle,
    ScriptScriptStyle,
}

#[derive(Debug, Clone, PartialEq)]
/// The body of an `Op` node: a symbol, a named operator, or an argument body.
pub enum OperatorContent {
    SymbolOperator(String),
    BodyOperator(Vec<ParseNode>),
    NamedOperator(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Horizontal alignment of a `Lap` (mathllap/mathrlap/mathclap) node.
pub enum LapAlignment {
    LLap,
    RLap,
    CLap,
}

#[derive(Debug, Clone, PartialEq)]
/// A column of an array environment: an alignment cell or a vertical separator.
pub enum ArrayColumn {
    AlignColumn {
        alignment: String,
        pre_gap: Option<f64>,
        post_gap: Option<f64>,
    },
    SeparatorColumn {
        separator: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// How columns of an array are separated.
pub enum ColumnSeparationType {
    AlignSeparation,
    AlignAtSeparation,
    GatherSeparation,
    SmallSeparation,
    CdSeparation,
}

#[derive(Debug, Clone, PartialEq)]
/// A parsed node in the LaTeX AST.
pub enum ParseNode {
    /// A command rejected by the trust policy. HTML displays the command in
    /// `color`; Unicode retains the established plain-text `fallback`.
    Unsupported {
        mode: Mode,
        command: String,
        color: String,
        fallback: Box<ParseNode>,
    },
    Internal {
        mode: Mode,
    },
    Raw {
        mode: Mode,
        string: String,
    },
    ColorToken {
        mode: Mode,
        color: String,
    },
    Size {
        mode: Mode,
        value: Measurement,
        is_blank: bool,
    },
    Url {
        mode: Mode,
        url: String,
    },
    Styling {
        mode: Mode,
        body: Vec<ParseNode>,
        style: StyleLevel,
        reset_font: bool,
    },
    Sqrt {
        mode: Mode,
        body: Box<ParseNode>,
        index: Option<Box<ParseNode>>,
    },
    Infix {
        mode: Mode,
        replace_with: String,
        size: Option<Measurement>,
        loc: Option<SourceLocation>,
    },
    GenFrac {
        mode: Mode,
        numer: Box<ParseNode>,
        denom: Box<ParseNode>,
        continued: bool,
        has_bar_line: bool,
        bar_size: Option<Measurement>,
        left_delim: Option<String>,
        right_delim: Option<String>,
    },
    Text {
        mode: Mode,
        body: Vec<ParseNode>,
        font: String,
    },
    Font {
        mode: Mode,
        font: String,
        body: Box<ParseNode>,
    },
    MClass {
        mode: Mode,
        mclass: AtomFamily,
        body: Vec<ParseNode>,
        is_character_box: bool,
    },
    Op {
        mode: Mode,
        limits: bool,
        always_handle_sup_sub: bool,
        parent_is_sup_sub: bool,
        suppress_base_shift: bool,
        content: OperatorContent,
    },
    OperatorName {
        mode: Mode,
        body: Vec<ParseNode>,
        always_handle_sup_sub: bool,
        limits: bool,
        parent_is_sup_sub: bool,
    },
    Overline {
        mode: Mode,
        body: Box<ParseNode>,
    },
    Underline {
        mode: Mode,
        body: Box<ParseNode>,
    },
    Smash {
        mode: Mode,
        body: Box<ParseNode>,
        smash_height: bool,
        smash_depth: bool,
    },
    Phantom {
        mode: Mode,
        body: Vec<ParseNode>,
    },
    VPhantom {
        mode: Mode,
        body: Box<ParseNode>,
    },
    Pmb {
        mode: Mode,
        mclass: AtomFamily,
        body: Vec<ParseNode>,
    },
    VCenter {
        mode: Mode,
        body: Box<ParseNode>,
    },
    Rule {
        mode: Mode,
        shift: Option<Measurement>,
        width: Measurement,
        height: Measurement,
    },
    RaiseBox {
        mode: Mode,
        dy: Measurement,
        body: Box<ParseNode>,
    },
    ReflectBox {
        mode: Mode,
        body: Box<ParseNode>,
    },
    HBox {
        mode: Mode,
        body: Vec<ParseNode>,
    },
    Lap {
        mode: Mode,
        alignment: LapAlignment,
        body: Box<ParseNode>,
    },
    MathChoice {
        mode: Mode,
        display: Vec<ParseNode>,
        text: Vec<ParseNode>,
        script: Vec<ParseNode>,
        scriptscript: Vec<ParseNode>,
    },
    Sizing {
        mode: Mode,
        size: usize,
        body: Vec<ParseNode>,
    },
    HorizBrace {
        mode: Mode,
        label: String,
        is_over: bool,
        base: Box<ParseNode>,
    },
    XArrow {
        mode: Mode,
        label: String,
        body: Box<ParseNode>,
        below: Option<Box<ParseNode>>,
    },
    AccentUnder {
        mode: Mode,
        label: String,
        base: Box<ParseNode>,
    },
    DelimSizing {
        mode: Mode,
        size: usize,
        mclass: AtomFamily,
        delim: String,
    },
    LeftRightRight {
        mode: Mode,
        delim: String,
        color: Option<String>,
    },
    LeftRight {
        mode: Mode,
        body: Vec<ParseNode>,
        left: String,
        right: String,
        right_color: Option<String>,
    },
    Middle {
        mode: Mode,
        delim: String,
    },
    Kern {
        mode: Mode,
        dimension: Measurement,
    },
    Enclose {
        mode: Mode,
        body: Box<ParseNode>,
        label: String,
        background_color: Option<String>,
        border_color: Option<String>,
    },
    Href {
        mode: Mode,
        href: String,
        body: Vec<ParseNode>,
    },
    Html {
        mode: Mode,
        /// Attributes in insertion order, matching KaTeX's HTML serialization.
        attributes: Vec<(String, String)>,
        body: Vec<ParseNode>,
    },
    IncludeGraphics {
        mode: Mode,
        alt: String,
        width: Measurement,
        height: Measurement,
        totalheight: Measurement,
        src: String,
    },
    Tag {
        mode: Mode,
        body: Vec<ParseNode>,
        tag: Vec<ParseNode>,
    },
    Array {
        mode: Mode,
        body: Vec<Vec<ParseNode>>,
        add_jot: bool,
        array_stretch: f64,
        columns: Option<Vec<ArrayColumn>>,
        row_gaps: Vec<Option<Measurement>>,
        hskip_before_and_after: bool,
        hlines_before_row: Vec<Vec<bool>>,
        column_separation_type: Option<ColumnSeparationType>,
        tags: Option<Vec<Option<Vec<ParseNode>>>>,
        auto_tags: Option<Vec<bool>>,
        leqno: bool,
    },
    EnvironmentEnd {
        mode: Mode,
        name: String,
    },
    CdLabel {
        mode: Mode,
        side: String,
        label: Box<ParseNode>,
    },
    CdParent {
        mode: Mode,
        fragment: Box<ParseNode>,
    },
    Cr {
        mode: Mode,
        new_line: bool,
        size: Option<Measurement>,
    },
    HtmlMathML {
        mode: Mode,
        html: Vec<ParseNode>,
        mathml: Vec<ParseNode>,
    },
    OrdGroup {
        mode: Mode,
        loc: Option<SourceLocation>,
        body: Vec<ParseNode>,
        semisimple: bool,
    },
    SupSub {
        mode: Mode,
        base: Option<Box<ParseNode>>,
        sup: Option<Box<ParseNode>>,
        sub: Option<Box<ParseNode>>,
    },
    TextOrd {
        mode: Mode,
        loc: Option<SourceLocation>,
        text: String,
    },
    MathOrd {
        mode: Mode,
        loc: Option<SourceLocation>,
        text: String,
    },
    Spacing {
        mode: Mode,
        loc: Option<SourceLocation>,
        text: String,
    },
    AccentToken {
        mode: Mode,
        loc: Option<SourceLocation>,
        text: String,
    },
    OperatorToken {
        mode: Mode,
        loc: Option<SourceLocation>,
        text: String,
    },
    Accent {
        mode: Mode,
        loc: Option<SourceLocation>,
        label: String,
        is_stretchy: bool,
        is_shifty: bool,
        base: Box<ParseNode>,
    },
    Verb {
        mode: Mode,
        loc: Option<SourceLocation>,
        body: String,
        star: bool,
    },
    Atom {
        mode: Mode,
        loc: Option<SourceLocation>,
        family: AtomFamily,
        text: String,
    },
    Color {
        mode: Mode,
        color: String,
        body: Vec<ParseNode>,
    },
}

impl ParseNode {
    /// Returns the [`Mode`] this node was parsed in.
    pub fn mode(&self) -> Mode {
        match self {
            ParseNode::Unsupported { mode, .. }
            | ParseNode::Internal { mode }
            | ParseNode::Raw { mode, .. }
            | ParseNode::ColorToken { mode, .. }
            | ParseNode::Size { mode, .. }
            | ParseNode::Url { mode, .. }
            | ParseNode::Styling { mode, .. }
            | ParseNode::Sqrt { mode, .. }
            | ParseNode::Infix { mode, .. }
            | ParseNode::GenFrac { mode, .. }
            | ParseNode::Text { mode, .. }
            | ParseNode::Font { mode, .. }
            | ParseNode::MClass { mode, .. }
            | ParseNode::Op { mode, .. }
            | ParseNode::OperatorName { mode, .. }
            | ParseNode::Overline { mode, .. }
            | ParseNode::Underline { mode, .. }
            | ParseNode::Smash { mode, .. }
            | ParseNode::Phantom { mode, .. }
            | ParseNode::VPhantom { mode, .. }
            | ParseNode::Pmb { mode, .. }
            | ParseNode::VCenter { mode, .. }
            | ParseNode::Rule { mode, .. }
            | ParseNode::RaiseBox { mode, .. }
            | ParseNode::ReflectBox { mode, .. }
            | ParseNode::HBox { mode, .. }
            | ParseNode::Lap { mode, .. }
            | ParseNode::MathChoice { mode, .. }
            | ParseNode::Sizing { mode, .. }
            | ParseNode::HorizBrace { mode, .. }
            | ParseNode::XArrow { mode, .. }
            | ParseNode::AccentUnder { mode, .. }
            | ParseNode::DelimSizing { mode, .. }
            | ParseNode::LeftRightRight { mode, .. }
            | ParseNode::LeftRight { mode, .. }
            | ParseNode::Middle { mode, .. }
            | ParseNode::Kern { mode, .. }
            | ParseNode::Enclose { mode, .. }
            | ParseNode::Href { mode, .. }
            | ParseNode::Html { mode, .. }
            | ParseNode::IncludeGraphics { mode, .. }
            | ParseNode::Tag { mode, .. }
            | ParseNode::Array { mode, .. }
            | ParseNode::EnvironmentEnd { mode, .. }
            | ParseNode::CdLabel { mode, .. }
            | ParseNode::CdParent { mode, .. }
            | ParseNode::Cr { mode, .. }
            | ParseNode::HtmlMathML { mode, .. }
            | ParseNode::OrdGroup { mode, .. }
            | ParseNode::SupSub { mode, .. }
            | ParseNode::TextOrd { mode, .. }
            | ParseNode::MathOrd { mode, .. }
            | ParseNode::Spacing { mode, .. }
            | ParseNode::AccentToken { mode, .. }
            | ParseNode::OperatorToken { mode, .. }
            | ParseNode::Accent { mode, .. }
            | ParseNode::Verb { mode, .. }
            | ParseNode::Atom { mode, .. }
            | ParseNode::Color { mode, .. } => *mode,
        }
    }
}
