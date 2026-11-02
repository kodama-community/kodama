use std::sync::Arc;

use crate::source_location::SourceLocation;

#[derive(Debug, Clone, PartialEq)]
/// A lexed or macro-expanded token.
pub struct Token {
    pub text: String,
    pub loc: Option<SourceLocation>,
    pub noexpand: bool,
    pub treat_as_relax: bool,
}

impl Token {
    pub fn new(text: impl Into<String>, loc: Option<SourceLocation>) -> Self {
        Token {
            text: text.into(),
            loc,
            noexpand: false,
            treat_as_relax: false,
        }
    }

    pub fn eof(input: impl Into<Arc<str>>, offset: usize) -> Self {
        let input = input.into();
        Token::new("EOF", Some(SourceLocation::new(input, offset, offset)))
    }

    pub fn range(&self, end_token: &Token, text: impl Into<String>) -> Self {
        if let (Some(start_loc), Some(end_loc)) = (&self.loc, &end_token.loc) {
            Token::new(text, Some(SourceLocation::range(start_loc, end_loc)))
        } else {
            Token::new(text, None)
        }
    }
}

/// Returns the source location of a token, if any.
pub fn token_location(token: Option<&Token>) -> Option<SourceLocation> {
    token.and_then(|token| token.loc.clone())
}
