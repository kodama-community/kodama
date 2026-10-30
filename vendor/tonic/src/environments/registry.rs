use std::collections::HashMap;

use crate::ast::{ArrayColumn, ColumnSeparationType, Mode, ParseNode, StyleLevel};
use crate::error::ParseError;
use crate::function_registry::ArgType;
use crate::functions::require_function_arg;

use super::alignat::alignat_environment_handler;
use super::alignment::{aligned_environment_handler, gather_environment_handler};
use super::cases::cases_environment_handler;
use super::cd::cd_environment_handler;
use super::equation::equation_environment_handler;
use super::matrix::{matrix_environment_handler, smallmatrix_environment_handler};
use super::subarray::subarray_environment_handler;

#[derive(Debug, Clone)]
/// Options controlling how an array environment is parsed.
pub struct ArrayEnvironmentOptions {
    pub columns: Option<Vec<ArrayColumn>>,
    pub array_stretch: Option<f64>,
    pub hskip_before_and_after: bool,
    pub cell_style: StyleLevel,
    pub max_columns: Option<usize>,
    pub single_row: bool,
    pub empty_single_row: bool,
    pub auto_tag: Option<bool>,
    pub leqno: bool,
    pub add_jot: bool,
    pub column_separation_type: Option<ColumnSeparationType>,
}

/// Callbacks the parser exposes to environment handlers.
pub trait EnvironmentParser {
    fn parse_array(&mut self, options: ArrayEnvironmentOptions) -> Result<ParseNode, ParseError>;
    fn parse_matrix_alignment(&mut self) -> Result<Option<String>, ParseError>;
    fn parse_cd(&mut self) -> Result<ParseNode, ParseError>;
}

/// Per-environment-call context: static data about the environment.
#[derive(Debug, Clone)]
pub struct EnvironmentContext {
    pub mode: Mode,
    pub display_mode: bool,
    pub leqno: bool,
    pub env_name: String,
}

/// An environment handler implementation.
pub type EnvironmentHandler = fn(
    parser: &mut dyn EnvironmentParser,
    context: &EnvironmentContext,
    args: &[ParseNode],
    opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError>;

#[derive(Debug, Clone)]
/// The declaration of an environment (name, arguments, handler).
pub struct EnvironmentSpec {
    pub names: Vec<String>,
    pub num_args: usize,
    pub num_optional_args: usize,
    pub arg_types: Vec<ArgType>,
    pub handler: EnvironmentHandler,
}

impl Default for EnvironmentSpec {
    fn default() -> Self {
        EnvironmentSpec {
            names: Vec::new(),
            num_args: 0,
            num_optional_args: 0,
            arg_types: Vec::new(),
            handler: |_, _, _, _| {
                Err(ParseError::InternalInvariant {
                    message: "Environment handler not set".to_string(),
                })
            },
        }
    }
}

pub fn array_columns(arg: &ParseNode, name: &str) -> Result<Vec<ArrayColumn>, ParseError> {
    let nodes = match arg {
        ParseNode::OrdGroup { body, .. } => body.clone(),
        _ => vec![arg.clone()],
    };
    let mut columns = Vec::new();
    for node in nodes {
        let text = match node {
            ParseNode::Atom { text, .. }
            | ParseNode::MathOrd { text, .. }
            | ParseNode::TextOrd { text, .. }
            | ParseNode::Spacing { text, .. } => text,
            _ => {
                return Err(ParseError::InvalidArgument {
                    message: format!("Unknown column alignment in {name}"),
                    loc: None,
                });
            }
        };
        match text.as_str() {
            "l" | "c" | "r" => columns.push(ArrayColumn::AlignColumn {
                alignment: text,
                pre_gap: None,
                post_gap: None,
            }),
            "|" | ":" => columns.push(ArrayColumn::SeparatorColumn { separator: text }),
            _ => {
                return Err(ParseError::InvalidArgument {
                    message: format!("Unknown column alignment: {text}"),
                    loc: None,
                });
            }
        }
    }
    Ok(columns)
}

fn array_environment_handler(
    parser: &mut dyn EnvironmentParser,
    context: &EnvironmentContext,
    args: &[ParseNode],
    _opt_args: &[Option<ParseNode>],
) -> Result<ParseNode, ParseError> {
    let columns = array_columns(
        &require_function_arg(args, 0, &format!("\\begin{{{}}}", context.env_name))?,
        &context.env_name,
    )?;
    parser.parse_array(ArrayEnvironmentOptions {
        columns: Some(columns.clone()),
        array_stretch: None,
        hskip_before_and_after: true,
        cell_style: if context.env_name == "darray" {
            StyleLevel::DisplayStyle
        } else {
            StyleLevel::TextStyle
        },
        max_columns: Some(columns.len()),
        single_row: false,
        empty_single_row: false,
        auto_tag: None,
        leqno: false,
        add_jot: false,
        column_separation_type: None,
    })
}

#[derive(Default)]
/// A map from environment names to their specs.
pub struct EnvironmentRegistry {
    entries: HashMap<String, EnvironmentSpec>,
}

impl EnvironmentRegistry {
    pub fn new() -> Self {
        EnvironmentRegistry {
            entries: HashMap::new(),
        }
    }

    pub fn register(&mut self, spec: EnvironmentSpec) {
        for name in &spec.names {
            self.entries.insert(name.clone(), spec.clone());
        }
    }

    pub fn get(&self, name: &str) -> Option<&EnvironmentSpec> {
        self.entries.get(name)
    }

    pub fn keys(&self) -> Vec<String> {
        self.entries.keys().cloned().collect()
    }
}

pub fn builtin_environment_specs() -> Vec<EnvironmentSpec> {
    vec![
        EnvironmentSpec {
            names: vec!["array".to_string(), "darray".to_string()],
            num_args: 1,
            handler: array_environment_handler,
            ..Default::default()
        },
        EnvironmentSpec {
            names: vec![
                "matrix".to_string(),
                "pmatrix".to_string(),
                "bmatrix".to_string(),
                "Bmatrix".to_string(),
                "vmatrix".to_string(),
                "Vmatrix".to_string(),
                "matrix*".to_string(),
                "pmatrix*".to_string(),
                "bmatrix*".to_string(),
                "Bmatrix*".to_string(),
                "vmatrix*".to_string(),
                "Vmatrix*".to_string(),
            ],
            handler: matrix_environment_handler,
            ..Default::default()
        },
        EnvironmentSpec {
            names: vec!["smallmatrix".to_string()],
            handler: smallmatrix_environment_handler,
            ..Default::default()
        },
        EnvironmentSpec {
            names: vec![
                "cases".to_string(),
                "dcases".to_string(),
                "rcases".to_string(),
                "drcases".to_string(),
            ],
            handler: cases_environment_handler,
            ..Default::default()
        },
        EnvironmentSpec {
            names: vec!["equation".to_string(), "equation*".to_string()],
            handler: equation_environment_handler,
            ..Default::default()
        },
        EnvironmentSpec {
            names: vec![
                "aligned".to_string(),
                "align".to_string(),
                "align*".to_string(),
                "split".to_string(),
            ],
            handler: aligned_environment_handler,
            ..Default::default()
        },
        EnvironmentSpec {
            names: vec![
                "gathered".to_string(),
                "gather".to_string(),
                "gather*".to_string(),
            ],
            handler: gather_environment_handler,
            ..Default::default()
        },
        EnvironmentSpec {
            names: vec![
                "alignat".to_string(),
                "alignat*".to_string(),
                "alignedat".to_string(),
            ],
            num_args: 1,
            handler: alignat_environment_handler,
            ..Default::default()
        },
        EnvironmentSpec {
            names: vec!["subarray".to_string()],
            num_args: 1,
            handler: subarray_environment_handler,
            ..Default::default()
        },
        EnvironmentSpec {
            names: vec!["CD".to_string()],
            handler: cd_environment_handler,
            ..Default::default()
        },
    ]
}

/// Builds an environment registry from the builtin specs plus caller-provided
/// extension specs (which override builtins sharing the same name).
pub fn build_environment_registry(extra_specs: &[EnvironmentSpec]) -> EnvironmentRegistry {
    let mut registry = EnvironmentRegistry::new();
    for spec in builtin_environment_specs() {
        registry.register(spec);
    }
    for spec in extra_specs {
        registry.register(spec.clone());
    }
    registry
}
