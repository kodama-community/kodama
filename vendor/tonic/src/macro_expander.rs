use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::Mode;
use crate::error::ParseError;
use crate::function_registry::lookup_function;
use crate::lexer::{Lexer, LexerReporter};
use crate::macro_definition::{MacroArgument, MacroDefinition, MacroExpansion};
use crate::namespace::Namespace;
use crate::settings::Settings;
use crate::symbol_registry::is_registered_symbol;
use crate::token::Token;

use crate::builtin_macros::{
    MacroReporter, MathSymbolGroupResolver, builtin_dynamic_macros, default_macro_reporter,
    default_math_symbol_group,
};
use crate::builtin_macros_static::builtin_static_macros;

#[derive(Debug, Clone)]
pub(crate) enum MacroReplacement {
    ReplacementText(String),
    ReplacementExpansion(MacroExpansion),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExpansionStep {
    NotExpanded,
    Expanded(usize),
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExternalCommandStatus {
    ExternalUndefined,
    ExternalExpandable,
    ExternalUnexpandable,
}

pub(crate) type CommandStatusResolver = Rc<dyn Fn(&str) -> ExternalCommandStatus>;

pub(crate) type MacroHandler =
    Rc<dyn Fn(&mut MacroExpander) -> Result<MacroReplacement, ParseError>>;

#[allow(dead_code)]
pub(crate) fn default_command_status(name: &str) -> ExternalCommandStatus {
    match lookup_function(name) {
        Some(spec) => {
            if spec.is_expandable() {
                ExternalCommandStatus::ExternalExpandable
            } else {
                ExternalCommandStatus::ExternalUnexpandable
            }
        }
        None => {
            if is_registered_symbol(name) {
                ExternalCommandStatus::ExternalUnexpandable
            } else {
                ExternalCommandStatus::ExternalUndefined
            }
        }
    }
}

pub(crate) struct MacroExpander {
    pub(crate) settings: Settings,
    expansion_count: usize,
    lexer: Lexer,
    pub(crate) macros: Namespace<MacroDefinition>,
    pub(crate) dynamic_macros: Namespace<MacroHandler>,
    stack: Vec<Token>,
    pub(crate) mode: Mode,
    pub(crate) report_nonstrict: LexerReporter,
    command_status: CommandStatusResolver,
    pub(crate) math_symbol_group: MathSymbolGroupResolver,
    pub(crate) macro_reporter: MacroReporter,
}

impl MacroExpander {
    pub(crate) fn new(
        input: &str,
        settings: Settings,
        report_nonstrict: LexerReporter,
        command_status: CommandStatusResolver,
    ) -> Self {
        let mut initial_macros = settings.macro_definitions();
        if let Some(macros) = &settings.macro_store {
            for (name, definition) in &macros.0 {
                initial_macros.insert(name.clone(), definition.clone());
            }
        }
        MacroExpander {
            settings,
            expansion_count: 0,
            lexer: Lexer::new(input, report_nonstrict.clone()),
            macros: Namespace::new(builtin_static_macros(), initial_macros),
            dynamic_macros: Namespace::new(builtin_dynamic_macros(), HashMap::new()),
            stack: Vec::new(),
            mode: Mode::Math,
            report_nonstrict,
            command_status,
            math_symbol_group: Rc::new(default_math_symbol_group),
            macro_reporter: Rc::new(default_macro_reporter),
        }
    }

    pub(crate) fn set_lexer_catcode(&mut self, char: &str, code: u8) {
        self.lexer.set_catcode(char, code);
    }

    #[allow(dead_code)]
    pub(crate) fn feed(&mut self, input: &str) {
        self.lexer = Lexer::new(input, self.report_nonstrict.clone());
    }

    pub(crate) fn switch_mode(&mut self, mode: Mode) {
        if self.mode != mode {
            self.mode = mode;
        }
    }

    pub(crate) fn begin_group(&mut self) {
        self.macros.begin_group();
        self.dynamic_macros.begin_group();
    }

    pub(crate) fn end_group(&mut self) -> Result<(), ParseError> {
        self.dynamic_macros.end_group()?;
        self.macros.end_group()
    }

    pub(crate) fn end_groups(&mut self) {
        self.macros.end_groups();
        self.dynamic_macros.end_groups();
    }

    pub(crate) fn future(&mut self) -> Result<Token, ParseError> {
        if self.stack.is_empty() {
            let token = self.lexer.lex()?;
            self.stack.push(token.clone());
            Ok(token)
        } else {
            self.stack
                .last()
                .cloned()
                .ok_or_else(|| ParseError::InternalInvariant {
                    message: "Empty token stack".to_string(),
                })
        }
    }

    pub(crate) fn pop_token(&mut self) -> Result<Token, ParseError> {
        let _ = self.future()?;
        self.stack
            .pop()
            .ok_or_else(|| ParseError::InternalInvariant {
                message: "Empty token stack".to_string(),
            })
    }

    pub(crate) fn push_token(&mut self, token: Token) {
        self.stack.push(token);
    }

    pub(crate) fn push_tokens(&mut self, tokens: Vec<Token>) {
        self.stack.extend(tokens);
    }

    pub(crate) fn consume_spaces(&mut self) -> Result<(), ParseError> {
        loop {
            let token = self.future()?;
            if token.text != " " {
                break;
            }
            self.pop_token()?;
        }
        Ok(())
    }

    pub(crate) fn scan_argument(&mut self, optional: bool) -> Result<Option<Token>, ParseError> {
        if optional {
            self.consume_spaces()?;
            if self.future()?.text != "[" {
                Ok(None)
            } else {
                let start = self.pop_token()?;
                let arg = self.consume_arg(Some(&["]".to_string()]))?;
                Ok(Some(self.push_argument_job(start, arg)))
            }
        } else {
            let arg = self.consume_arg(None)?;
            Ok(Some(self.push_argument_job(arg.start.clone(), arg)))
        }
    }

    fn push_argument_job(&mut self, start: Token, arg: MacroArgument) -> Token {
        let eof = match &arg.end.loc {
            Some(loc) => Token::new("EOF", Some(loc.clone())),
            None => Token::new("EOF", None),
        };
        self.push_token(eof);
        self.push_tokens(arg.tokens);
        start.range(&arg.end, "")
    }

    pub(crate) fn consume_arg(
        &mut self,
        delimiters: Option<&[String]>,
    ) -> Result<MacroArgument, ParseError> {
        let is_delimited = delimiters.is_some_and(|values| !values.is_empty());
        if !is_delimited {
            self.consume_spaces()?;
        }
        let start = self.future()?;
        let mut tokens: Vec<Token> = Vec::new();
        let mut depth: i64 = 0;
        let mut delimiter_match = 0;
        loop {
            let token = self.pop_token()?;
            tokens.push(token.clone());
            if token.text == "{" {
                depth += 1;
            } else if token.text == "}" {
                depth -= 1;
                if depth == -1 {
                    return Err(ParseError::InvalidArgument {
                        message: "Extra }".to_string(),
                        loc: token.loc.clone(),
                    });
                }
            } else if token.text == "EOF" {
                return Err(ParseError::InvalidArgument {
                    message: format!(
                        "Unexpected end of input in a macro argument, expected '{}'",
                        expected_argument_delimiter(delimiters, delimiter_match)
                    ),
                    loc: token.loc.clone(),
                });
            }
            if let Some(values) = delimiters {
                if is_delimited {
                    if delimiter_is_active(values, delimiter_match, depth, &token) {
                        delimiter_match += 1;
                        if delimiter_match == values.len() {
                            for _ in 0..delimiter_match {
                                tokens.pop();
                            }
                            let normalized = normalize_consumed_argument(&start, tokens);
                            return Ok(MacroArgument {
                                start,
                                end: token,
                                tokens: normalized,
                            });
                        }
                    } else {
                        delimiter_match = 0;
                    }
                }
            }
            if depth == 0 && !is_delimited {
                let normalized = normalize_consumed_argument(&start, tokens);
                return Ok(MacroArgument {
                    start,
                    end: token,
                    tokens: normalized,
                });
            }
        }
    }

    pub(crate) fn consume_args(
        &mut self,
        num_args: usize,
        delimiters: Option<&[Vec<String>]>,
    ) -> Result<Vec<Vec<Token>>, ParseError> {
        if let Some(values) = delimiters {
            if values.len() != num_args + 1 {
                return Err(ParseError::InvalidArgument {
                    message: "The length of delimiters doesn't match the number of args!"
                        .to_string(),
                    loc: None,
                });
            }
            if let Some(prefix) = values.first() {
                for expected in prefix {
                    let token = self.pop_token()?;
                    if &token.text != expected {
                        return Err(ParseError::InvalidArgument {
                            message: "Use of the macro doesn't match its definition".to_string(),
                            loc: token.loc.clone(),
                        });
                    }
                }
            }
        }
        let mut args: Vec<Vec<Token>> = Vec::new();
        for index in 0..num_args {
            let argument_delimiters = delimiters
                .and_then(|values| values.get(index + 1))
                .map(|v| &v[..]);
            args.push(self.consume_arg(argument_delimiters)?.tokens);
        }
        Ok(args)
    }

    pub(crate) fn count_expansion(&mut self, amount: usize) -> Result<(), ParseError> {
        self.expansion_count += amount;
        if self.expansion_count > self.settings.max_expand {
            return Err(ParseError::TooManyExpansions {
                limit: self.settings.max_expand,
            });
        }
        Ok(())
    }

    pub(crate) fn expand_once(
        &mut self,
        expandable_only: bool,
    ) -> Result<ExpansionStep, ParseError> {
        let top_token = self.pop_token()?;
        let expansion = if top_token.noexpand {
            None
        } else {
            self.get_expansion(&top_token.text)?
        };
        match expansion {
            None => {
                if expandable_only
                    && !top_token.text.is_empty()
                    && top_token.text.starts_with('\\')
                    && !self.is_defined(&top_token.text)
                {
                    Err(ParseError::UndefinedControlSequence {
                        name: top_token.text,
                        loc: None,
                    })
                } else {
                    self.push_token(top_token);
                    Ok(ExpansionStep::NotExpanded)
                }
            }
            Some(value) => {
                if expandable_only && value.unexpandable {
                    self.push_token(top_token);
                    Ok(ExpansionStep::NotExpanded)
                } else {
                    self.count_expansion(1)?;
                    let args = self.consume_args(value.num_args, value.delimiters.as_deref())?;
                    let tokens = if value.num_args == 0 {
                        value.tokens
                    } else {
                        substitute_macro_arguments(&value.tokens, &args)?
                    };
                    let len = tokens.len();
                    self.push_tokens(tokens);
                    Ok(ExpansionStep::Expanded(len))
                }
            }
        }
    }

    pub(crate) fn expand_after_future(&mut self) -> Result<Token, ParseError> {
        self.expand_once(false)?;
        self.future()
    }

    pub(crate) fn expand_next_token(&mut self) -> Result<Token, ParseError> {
        loop {
            match self.expand_once(false)? {
                ExpansionStep::Expanded(_) => continue,
                ExpansionStep::NotExpanded => {
                    let mut token = self.pop_token()?;
                    if token.treat_as_relax {
                        token.text = "\\relax".to_string();
                    }
                    return Ok(token);
                }
            }
        }
    }

    pub(crate) fn expand_tokens(&mut self, tokens: Vec<Token>) -> Result<Vec<Token>, ParseError> {
        let mut output: Vec<Token> = Vec::new();
        let old_stack_length = self.stack.len();
        self.push_tokens(tokens);
        while self.stack.len() > old_stack_length {
            match self.expand_once(true)? {
                ExpansionStep::Expanded(_) => (),
                ExpansionStep::NotExpanded => {
                    let mut token = self.pop_token()?;
                    if token.treat_as_relax {
                        token.noexpand = false;
                        token.treat_as_relax = false;
                    }
                    output.push(token);
                }
            }
        }
        self.count_expansion(output.len())?;
        Ok(output)
    }

    #[allow(dead_code)]
    pub(crate) fn expand_macro(&mut self, name: &str) -> Result<Option<Vec<Token>>, ParseError> {
        if !self.macros.has(name) && !self.dynamic_macros.has(name) {
            Ok(None)
        } else {
            Ok(Some(self.expand_tokens(vec![Token::new(name, None)])?))
        }
    }

    #[allow(dead_code)]
    pub(crate) fn expand_macro_as_text(
        &mut self,
        name: &str,
    ) -> Result<Option<String>, ParseError> {
        Ok(self
            .expand_macro(name)?
            .map(|tokens| tokens.iter().map(|token| token.text.clone()).collect()))
    }

    fn lex_macro_body(&mut self, expansion: &str) -> Result<MacroExpansion, ParseError> {
        let mut lexer = Lexer::new(expansion, self.report_nonstrict.clone());
        let mut tokens: Vec<Token> = Vec::new();
        loop {
            let token = lexer.lex()?;
            if token.text == "EOF" {
                tokens.reverse();
                break Ok(MacroExpansion {
                    tokens,
                    num_args: inferred_argument_count(expansion),
                    delimiters: None,
                    unexpandable: false,
                });
            }
            tokens.push(token);
            continue;
        }
    }

    fn resolve_replacement(
        &mut self,
        replacement: MacroReplacement,
    ) -> Result<MacroExpansion, ParseError> {
        match replacement {
            MacroReplacement::ReplacementText(text) => self.lex_macro_body(&text),
            MacroReplacement::ReplacementExpansion(expansion) => Ok(expansion),
        }
    }

    fn get_expansion(&mut self, name: &str) -> Result<Option<MacroExpansion>, ParseError> {
        if name.len() == 1 && self.lexer.catcode(name).is_some_and(|code| code != 13) {
            return Ok(None);
        }
        match self.dynamic_macros.get_current(name) {
            Some(handler) => {
                let handler = handler.clone();
                let replacement = handler(self)?;
                Ok(Some(self.resolve_replacement(replacement)?))
            }
            None => self.get_static_or_builtin_dynamic_expansion(name),
        }
    }

    fn get_static_or_builtin_dynamic_expansion(
        &mut self,
        name: &str,
    ) -> Result<Option<MacroExpansion>, ParseError> {
        match self.macros.get(name) {
            Some(definition) => match definition {
                MacroDefinition::Text(expansion) => {
                    let expansion = expansion.clone();
                    Ok(Some(self.lex_macro_body(&expansion)?))
                }
                MacroDefinition::Expansion(expansion) => Ok(Some(expansion.clone())),
            },
            None => match self.dynamic_macros.get_builtin(name) {
                Some(handler) => {
                    let handler = handler.clone();
                    let replacement = handler(self)?;
                    Ok(Some(self.resolve_replacement(replacement)?))
                }
                None => Ok(None),
            },
        }
    }

    pub(crate) fn is_defined(&self, name: &str) -> bool {
        let external_defined = match (self.command_status)(name) {
            ExternalCommandStatus::ExternalUndefined => false,
            ExternalCommandStatus::ExternalExpandable
            | ExternalCommandStatus::ExternalUnexpandable => true,
        };
        self.macros.has(name)
            || self.dynamic_macros.has(name)
            || external_defined
            || is_implicit_command(name)
    }

    pub(crate) fn is_expandable(&self, name: &str) -> bool {
        self.dynamic_macros.get_current(name).is_some()
            || self.is_static_or_builtin_dynamic_expandable(name)
    }

    fn is_static_or_builtin_dynamic_expandable(&self, name: &str) -> bool {
        match self.macros.get(name) {
            Some(definition) => match definition {
                MacroDefinition::Text(_) => true,
                MacroDefinition::Expansion(value) => !value.unexpandable,
            },
            None => {
                if self.dynamic_macros.get_builtin(name).is_some() {
                    true
                } else {
                    match (self.command_status)(name) {
                        ExternalCommandStatus::ExternalExpandable => true,
                        ExternalCommandStatus::ExternalUndefined
                        | ExternalCommandStatus::ExternalUnexpandable => false,
                    }
                }
            }
        }
    }
}

fn delimiter_is_active(
    delimiters: &[String],
    match_index: usize,
    depth: i64,
    token: &Token,
) -> bool {
    delimiters.get(match_index).is_some_and(|delimiter| {
        (depth == 0 || (depth == 1 && delimiter == "{")) && token.text == *delimiter
    })
}

fn expected_argument_delimiter(delimiters: Option<&[String]>, match_index: usize) -> String {
    delimiters.map_or_else(
        || "}".to_string(),
        |values| {
            values
                .get(match_index)
                .cloned()
                .unwrap_or_else(|| "}".to_string())
        },
    )
}

fn normalize_consumed_argument(start: &Token, tokens: Vec<Token>) -> Vec<Token> {
    if start.text == "{" && tokens.last().is_some_and(|last| last.text == "}") {
        let mut without = tokens[1..tokens.len() - 1].to_vec();
        without.reverse();
        without
    } else {
        let mut tokens = tokens;
        tokens.reverse();
        tokens
    }
}

fn placeholder_number(text: &str) -> Option<usize> {
    let mut chars = text.chars();
    let c = chars.next()?;
    if chars.next().is_none() && c.is_ascii_digit() && c >= '1' {
        Some((c as u32 - '0' as u32) as usize)
    } else {
        None
    }
}

fn substitute_macro_arguments(
    tokens: &[Token],
    args: &[Vec<Token>],
) -> Result<Vec<Token>, ParseError> {
    let forward: Vec<Token> = tokens.iter().rev().cloned().collect();
    let mut output: Vec<Token> = Vec::new();
    let mut index = 0;
    while index < forward.len() {
        let token = &forward[index];
        if token.text != "#" {
            output.push(token.clone());
            index += 1;
            continue;
        }
        let Some(next) = forward.get(index + 1) else {
            return Err(ParseError::InvalidArgument {
                message: "Incomplete placeholder at end of macro body".to_string(),
                loc: token.loc.clone(),
            });
        };
        if next.text == "#" {
            output.push(next.clone());
            index += 2;
            continue;
        }
        let Some(number) = placeholder_number(&next.text) else {
            return Err(ParseError::InvalidArgument {
                message: "Not a valid argument number".to_string(),
                loc: next.loc.clone(),
            });
        };
        let Some(argument) = args.get(number - 1) else {
            return Err(ParseError::InvalidArgument {
                message: "Not a valid argument number".to_string(),
                loc: next.loc.clone(),
            });
        };
        output.extend(argument.iter().rev().cloned());
        index += 2;
    }
    output.reverse();
    Ok(output)
}

fn inferred_argument_count(expansion: &str) -> usize {
    let chars: Vec<char> = expansion.chars().collect();
    let mut seen = [false; 10];
    let mut index = 0;
    while index < chars.len() {
        if chars[index] != '#' || index + 1 >= chars.len() {
            index += 1;
            continue;
        }
        if chars[index + 1] == '#' {
            index += 2;
            continue;
        }
        let digit = chars[index + 1];
        if digit.is_ascii_digit() && digit >= '1' {
            seen[(digit as u32 - '0' as u32) as usize] = true;
            index += 2;
            continue;
        }
        index += 1;
    }
    let mut count = 0;
    while count < 9 {
        if !seen[count + 1] {
            break;
        }
        count += 1;
    }
    count
}

pub(crate) fn is_implicit_command(name: &str) -> bool {
    matches!(name, "^" | "_" | "\\limits" | "\\nolimits")
}

pub(crate) fn token_expansion(tokens: Vec<Token>) -> MacroReplacement {
    MacroReplacement::ReplacementExpansion(MacroExpansion {
        tokens,
        num_args: 0,
        delimiters: None,
        unexpandable: false,
    })
}
