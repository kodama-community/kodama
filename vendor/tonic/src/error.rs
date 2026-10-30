use crate::source_location::SourceLocation;
use crate::token::Token;

/// A parse failure, mirroring MoonBit's `ParseFailure` suberror.
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    UnexpectedCharacter {
        message: String,
        loc: Option<SourceLocation>,
    },
    ExpectedToken {
        expected: String,
        actual: Diagnostic,
    },
    UndefinedControlSequence {
        name: String,
        loc: Option<SourceLocation>,
    },
    InvalidArgument {
        message: String,
        loc: Option<SourceLocation>,
    },
    DoubleSuperscript {
        loc: Option<SourceLocation>,
    },
    DoubleSubscript {
        loc: Option<SourceLocation>,
    },
    ExpectedGroupAfter {
        symbol: String,
        loc: Option<SourceLocation>,
    },
    FunctionNotAllowed {
        func_name: String,
        context: String,
        loc: Option<SourceLocation>,
    },
    MissingFunctionHandler {
        func_name: String,
        loc: Option<SourceLocation>,
    },
    TooManyExpansions {
        limit: usize,
    },
    InternalInvariant {
        message: String,
    },
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedCharacter { message, .. } => write!(f, "{message}"),
            ParseError::ExpectedToken { expected, actual } => {
                write!(f, "Expected token {expected:?}, got {:?}", actual.text)
            }
            ParseError::UndefinedControlSequence { name, .. } => {
                write!(f, "Undefined control sequence: {name}")
            }
            ParseError::InvalidArgument { message, .. } => write!(f, "{message}"),
            ParseError::DoubleSuperscript { .. } => write!(f, "Double superscript"),
            ParseError::DoubleSubscript { .. } => write!(f, "Double subscript"),
            ParseError::ExpectedGroupAfter { symbol, .. } => {
                write!(f, "Expected group after {symbol}")
            }
            ParseError::FunctionNotAllowed {
                func_name, context, ..
            } => write!(
                f,
                "Function {func_name} is not allowed in {context} context"
            ),
            ParseError::MissingFunctionHandler { func_name, .. } => {
                write!(f, "No handler defined for function {func_name}")
            }
            ParseError::TooManyExpansions { limit } => {
                write!(f, "Too many expansions: reached limit of {limit}")
            }
            ParseError::InternalInvariant { message } => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for ParseError {}

/// Diagnostic information attached to an offending token.
#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    pub text: String,
    pub loc: Option<SourceLocation>,
}

impl Diagnostic {
    pub fn from_token(token: &Token) -> Self {
        Diagnostic {
            text: token.text.clone(),
            loc: token.loc.clone(),
        }
    }
}
