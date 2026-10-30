use crate::token::Token;

#[derive(Debug, Clone, PartialEq)]
/// A macro expansion with its argument count and delimiters.
pub struct MacroExpansion {
    pub tokens: Vec<Token>,
    pub num_args: usize,
    pub delimiters: Option<Vec<Vec<String>>>,
    pub unexpandable: bool,
}

impl MacroExpansion {
    pub fn new(
        tokens: Vec<Token>,
        num_args: usize,
        delimiters: Option<Vec<Vec<String>>>,
        unexpandable: bool,
    ) -> Self {
        MacroExpansion {
            tokens,
            num_args,
            delimiters,
            unexpandable,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
/// A macro body: raw text or a pre-expanded token list.
pub enum MacroDefinition {
    Text(String),
    Expansion(MacroExpansion),
}

impl MacroDefinition {
    pub fn text(expansion: impl Into<String>) -> Self {
        MacroDefinition::Text(expansion.into())
    }

    pub fn expansion(expansion: MacroExpansion) -> Self {
        MacroDefinition::Expansion(expansion)
    }

    pub fn as_text(&self) -> Option<&str> {
        if let MacroDefinition::Text(text) = self {
            Some(text)
        } else {
            None
        }
    }
}

/// A consumed macro argument: the opening and closing tokens plus the
/// normalized inner token list.
pub(crate) struct MacroArgument {
    pub start: Token,
    pub end: Token,
    pub tokens: Vec<Token>,
}
