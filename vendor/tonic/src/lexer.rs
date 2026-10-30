use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use crate::error::ParseError;
use crate::source_location::SourceLocation;
use crate::token::Token;

pub type LexerReporter = Rc<dyn Fn(&str, &str) -> Result<(), ParseError>>;

pub(crate) struct Lexer {
    input: Arc<str>,
    chars: Vec<char>,
    offset: usize,
    catcodes: HashMap<String, u8>,
    report_nonstrict: LexerReporter,
}

impl Lexer {
    pub(crate) fn new(input: &str, report_nonstrict: LexerReporter) -> Self {
        let mut lexer = Lexer {
            input: Arc::from(input),
            chars: input.chars().collect(),
            offset: 0,
            catcodes: HashMap::new(),
            report_nonstrict,
        };
        lexer.set_catcode("%", 14);
        lexer.set_catcode("~", 13);
        lexer
    }

    pub(crate) fn set_catcode(&mut self, char: &str, code: u8) {
        self.catcodes.insert(char.to_string(), code);
    }

    pub(crate) fn catcode(&self, char: &str) -> Option<u8> {
        self.catcodes.get(char).copied()
    }

    pub(crate) fn lex(&mut self) -> Result<Token, ParseError> {
        loop {
            if self.offset == self.chars.len() {
                return Ok(Token::eof(self.input.clone(), self.offset));
            }
            let token = self.match_token()?;
            if self.catcodes.get(&token.text) == Some(&14) {
                if let Some(offset) = next_line_start(&self.chars, self.offset) {
                    self.offset = offset;
                    continue;
                }
                self.offset = self.chars.len();
                (self.report_nonstrict)(
                    "commentAtEnd",
                    "% comment has no terminating newline; LaTeX would fail because of commenting the end of math mode (e.g. $)",
                )?;
                continue;
            }
            return Ok(token);
        }
    }

    fn match_token(&mut self) -> Result<Token, ParseError> {
        let start = self.offset;
        let c = self.chars[start];
        if is_space_code_unit(c) {
            let end = whitespace_end(&self.chars, start + 1);
            self.offset = end;
            Ok(token_at(&self.input, start, end, " "))
        } else if c == '\\' {
            self.match_backslash_token(start)
        } else if is_regular_code_unit(c) || (c as u32) >= 0x10000 {
            let end = combining_marks_end(&self.chars, start + 1);
            self.offset = end;
            Ok(token_at(
                &self.input,
                start,
                end,
                token_text(&self.chars, start, end),
            ))
        } else {
            Err(unexpected_character(&self.input, start, c))
        }
    }

    fn match_backslash_token(&mut self, start: usize) -> Result<Token, ParseError> {
        if let Some(end) = control_space_end(&self.chars, start) {
            self.offset = end;
            return Ok(token_at(&self.input, start, end, "\\ "));
        }
        if let Some(end) = verb_end(&self.chars, start, true) {
            self.offset = end;
            return Ok(token_at(
                &self.input,
                start,
                end,
                token_text(&self.chars, start, end),
            ));
        }
        if let Some(end) = verb_end(&self.chars, start, false) {
            self.offset = end;
            return Ok(token_at(
                &self.input,
                start,
                end,
                token_text(&self.chars, start, end),
            ));
        }
        if let Some((raw_end, end)) = control_word_end(&self.chars, start) {
            self.offset = end;
            return Ok(token_at(
                &self.input,
                start,
                end,
                token_text(&self.chars, start, raw_end),
            ));
        }
        let len = self.chars.len();
        if start + 1 < len && (self.chars[start + 1] as u32) <= 0xffff {
            let end = start + 2;
            self.offset = end;
            return Ok(token_at(
                &self.input,
                start,
                end,
                token_text(&self.chars, start, end),
            ));
        }
        Err(unexpected_character(&self.input, start, self.chars[start]))
    }
}

fn is_space_code_unit(c: char) -> bool {
    c == ' ' || c == '\r' || c == '\n' || c == '\t'
}

fn is_horizontal_space_code_unit(c: char) -> bool {
    c == ' ' || c == '\r' || c == '\t'
}

fn is_ascii_letter(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '@'
}

pub(crate) fn is_ascii_alphabetic(c: char) -> bool {
    c.is_ascii_alphabetic()
}

pub(crate) fn is_combining_diacritical_mark(c: char) -> bool {
    ('\u{0300}'..='\u{036f}').contains(&c)
}

fn is_regular_code_unit(c: char) -> bool {
    ('\u{0021}'..='\u{005b}').contains(&c)
        || ('\u{005d}'..='\u{2027}').contains(&c)
        || ('\u{202a}'..='\u{d7ff}').contains(&c)
        || ('\u{f900}'..='\u{ffff}').contains(&c)
}

fn is_js_line_terminator(c: char) -> bool {
    c == '\n' || c == '\r' || c == '\u{2028}' || c == '\u{2029}'
}

fn token_text(chars: &[char], start: usize, end: usize) -> String {
    chars[start..end].iter().collect()
}

fn token_at(input: &Arc<str>, start: usize, end: usize, text: impl Into<String>) -> Token {
    Token::new(text, Some(SourceLocation::new(input.clone(), start, end)))
}

fn combining_marks_end(chars: &[char], offset: usize) -> usize {
    let mut i = offset;
    while i < chars.len() && is_combining_diacritical_mark(chars[i]) {
        i += 1;
    }
    i
}

fn whitespace_end(chars: &[char], offset: usize) -> usize {
    let mut i = offset;
    while i < chars.len() && is_space_code_unit(chars[i]) {
        i += 1;
    }
    i
}

fn horizontal_space_end(chars: &[char], offset: usize) -> usize {
    let mut i = offset;
    while i < chars.len() && is_horizontal_space_code_unit(chars[i]) {
        i += 1;
    }
    i
}

fn control_space_end(chars: &[char], start: usize) -> Option<usize> {
    let len = chars.len();
    if start + 1 >= len {
        None
    } else if chars[start + 1] == '\n' {
        Some(horizontal_space_end(chars, start + 2))
    } else if is_horizontal_space_code_unit(chars[start + 1]) {
        let spaces_end = horizontal_space_end(chars, start + 1);
        let after_newline = if spaces_end < len && chars[spaces_end] == '\n' {
            spaces_end + 1
        } else {
            spaces_end
        };
        Some(horizontal_space_end(chars, after_newline))
    } else {
        None
    }
}

pub(crate) fn starts_with_at(chars: &[char], offset: usize, prefix: &str) -> bool {
    let prefix_chars: Vec<char> = prefix.chars().collect();
    let prefix_len = prefix_chars.len();
    offset + prefix_len <= chars.len() && chars[offset..offset + prefix_len] == prefix_chars[..]
}

fn verb_end(chars: &[char], start: usize, starred: bool) -> Option<usize> {
    let prefix = if starred { "\\verb*" } else { "\\verb" };
    if !starts_with_at(chars, start, prefix) {
        return None;
    }
    let delimiter_offset = start + prefix.chars().count();
    if delimiter_offset >= chars.len() {
        return None;
    }
    let delimiter = chars[delimiter_offset];
    if !starred && (delimiter == '*' || is_ascii_alphabetic(delimiter)) {
        return None;
    }
    let mut i = delimiter_offset + 1;
    while i < chars.len() {
        let c = chars[i];
        if c == delimiter {
            return Some(i + 1);
        } else if is_js_line_terminator(c) {
            return None;
        }
        i += 1;
    }
    None
}

fn control_word_end(chars: &[char], start: usize) -> Option<(usize, usize)> {
    let len = chars.len();
    if start + 1 >= len || !is_ascii_letter(chars[start + 1]) {
        None
    } else {
        let mut raw_end = start + 2;
        while raw_end < len && is_ascii_letter(chars[raw_end]) {
            raw_end += 1;
        }
        Some((raw_end, whitespace_end(chars, raw_end)))
    }
}

fn next_line_start(chars: &[char], offset: usize) -> Option<usize> {
    chars[offset..]
        .iter()
        .position(|&c| c == '\n')
        .map(|p| offset + p + 1)
}

fn unexpected_character(input: &Arc<str>, offset: usize, c: char) -> ParseError {
    let text: String = c.to_string();
    ParseError::UnexpectedCharacter {
        message: format!("Unexpected character: '{text}'"),
        loc: Some(SourceLocation::new(input.clone(), offset, offset + 1)),
    }
}
