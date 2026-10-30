use std::rc::Rc;

use crate::ast::{AtomFamily, ColumnSeparationType, Measurement, Mode, ParseNode, StyleLevel};
use crate::environments::{
    build_environment_registry, cd_row, ArrayEnvironmentOptions, EnvironmentContext,
    EnvironmentParser, EnvironmentRegistry, EnvironmentSpec,
};
use crate::error::{Diagnostic, ParseError};
use crate::function_registry::{
    build_function_registry, ArgType, FunctionContext, FunctionParser, FunctionRegistry,
    FunctionSpec,
};
use crate::functions::{parse_size_measurement, size_scan_candidate, valid_size_unit};
use crate::lexer::{is_ascii_alphabetic, starts_with_at};
use crate::macro_definition::MacroDefinition;
use crate::macro_expander::{is_implicit_command, ExternalCommandStatus, MacroExpander};
use crate::settings::{Settings, TrustContext};
use crate::source_location::SourceLocation;
use crate::symbol_registry::{is_registered_symbol, lookup_symbol, SymbolGroup, SymbolSpec};
use crate::text_ligature::form_text_ligatures;
use crate::token::Token;
use crate::unicode_scripts::{lookup_unicode_script, supported_codepoint, UnicodeScriptKind};
use crate::unicode_symbols::{
    normalize_unicode_symbol, trailing_combining_mark_start, unicode_accent_command,
};

pub struct Parser {
    mode: Mode,
    gullet: MacroExpander,
    settings: Settings,
    function_registry: FunctionRegistry,
    environment_registry: EnvironmentRegistry,
    next_token: Option<Token>,
    leftright_depth: usize,
}

enum AtomResult {
    EmitAtom(ParseNode),
    SkipAtom,
}

impl Parser {
    pub(crate) fn new(
        input: &str,
        settings: Settings,
        extra_specs: &[FunctionSpec],
        extra_env_specs: &[EnvironmentSpec],
    ) -> Parser {
        let registry = build_function_registry(extra_specs);
        let command_registry = registry.clone();
        let settings_for_reporter = settings.clone();
        let gullet = MacroExpander::new(
            input,
            settings.clone(),
            Rc::new(move |error_code, error_message| {
                settings_for_reporter.report_nonstrict(error_code, error_message, None)
            }),
            Rc::new(move |name| match command_registry.get(name) {
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
            }),
        );
        Parser {
            mode: Mode::Math,
            gullet,
            settings,
            function_registry: registry,
            environment_registry: build_environment_registry(extra_env_specs),
            next_token: None,
            leftright_depth: 0,
        }
    }

    fn fetch(&mut self) -> Result<Token, ParseError> {
        match &self.next_token {
            Some(token) => Ok(token.clone()),
            None => {
                let token = self.gullet.expand_next_token()?;
                self.next_token = Some(token.clone());
                Ok(token)
            }
        }
    }

    fn consume(&mut self) {
        self.next_token = None;
    }

    fn expect(&mut self, text: &str, consume: bool) -> Result<(), ParseError> {
        let token = self.fetch()?;
        if token.text != text {
            return Err(ParseError::ExpectedToken {
                expected: text.to_string(),
                actual: Diagnostic::from_token(&token),
            });
        }
        if consume {
            self.consume();
        }
        Ok(())
    }

    fn parse(&mut self) -> Result<Vec<ParseNode>, ParseError> {
        if !self.settings.global_group {
            self.gullet.begin_group();
        }
        if self.settings.color_is_text_color {
            self.gullet.macros.set(
                "\\color".to_string(),
                Some(MacroDefinition::text("\\textcolor")),
                false,
            );
        }
        let result: Result<Vec<ParseNode>, ParseError> = (|| {
            let body = self.parse_expression(false, None)?;
            self.expect("EOF", true)?;
            Ok(body)
        })();
        let close_result = if self.settings.global_group {
            Ok(())
        } else {
            self.gullet.end_group()
        };
        self.gullet.end_groups();
        if self.settings.global_group {
            self.persist_user_macros();
        }
        unwrap_captured(close_result, result)
    }

    fn persist_user_macros(&mut self) {
        let user_entries = self.gullet.macros.get_user_entries();
        if let Some(macros) = &mut self.settings.macro_store {
            for (name, definition) in user_entries {
                macros.0.insert(name, definition.clone());
            }
        }
    }

    fn subparse(&mut self, tokens: Vec<Token>) -> Result<Vec<ParseNode>, ParseError> {
        let old_token = self.next_token.clone();
        self.consume();
        self.gullet.push_token(Token::new("}", None));
        self.gullet.push_tokens(tokens);
        let result = self.subparse_inner();
        self.next_token = old_token;
        result
    }

    fn subparse_inner(&mut self) -> Result<Vec<ParseNode>, ParseError> {
        let body = self.parse_expression(false, Some("}"))?;
        self.expect("}", true)?;
        Ok(body)
    }

    fn parse_math_mode(&mut self, close: &str) -> Result<Vec<ParseNode>, ParseError> {
        let outer_mode = self.mode;
        self.switch_mode(Mode::Math);
        let result = self.parse_math_mode_inner(close);
        self.switch_mode(outer_mode);
        result
    }

    fn parse_math_mode_inner(&mut self, close: &str) -> Result<Vec<ParseNode>, ParseError> {
        let body = self.parse_expression(false, Some(close))?;
        self.expect(close, true)?;
        Ok(body)
    }

    fn current_color(&self) -> Result<Option<String>, ParseError> {
        match self.gullet.macros.get("\\current@color") {
            None => Ok(None),
            Some(MacroDefinition::Text(color)) => Ok(Some(color.clone())),
            Some(MacroDefinition::Expansion(_)) => Err(ParseError::InvalidArgument {
                message: "\\current@color set to non-string in \\right".to_string(),
                loc: None,
            }),
        }
    }

    fn parse_left_right(&mut self, left: &str) -> Result<ParseNode, ParseError> {
        self.leftright_depth += 1;
        let result = self.parse_left_right_inner(left);
        self.leftright_depth -= 1;
        result
    }

    fn parse_left_right_inner(&mut self, left: &str) -> Result<ParseNode, ParseError> {
        let body = self.parse_expression(false, None)?;
        self.expect("\\right", false)?;
        let Some(AtomResult::EmitAtom(ParseNode::LeftRightRight {
            delim: right,
            color,
            ..
        })) = self.parse_function(None, None)?
        else {
            return Err(ParseError::InternalInvariant {
                message: "\\right did not produce a closing delimiter".to_string(),
            });
        };
        Ok(ParseNode::LeftRight {
            mode: self.mode,
            body,
            left: left.to_string(),
            right,
            right_color: color,
        })
    }

    fn parse_expression(
        &mut self,
        break_on_infix: bool,
        break_on_token_text: Option<&str>,
    ) -> Result<Vec<ParseNode>, ParseError> {
        let mut body: Vec<ParseNode> = Vec::new();
        loop {
            if self.mode == Mode::Math {
                self.consume_spaces()?;
            }
            let token = self.fetch()?;
            if self.should_break_expression(&token.text, break_on_infix, break_on_token_text) {
                return self.finish_expression(body);
            }
            match self.parse_atom(break_on_token_text)? {
                None => return self.finish_expression(body),
                Some(AtomResult::SkipAtom) => continue,
                Some(AtomResult::EmitAtom(node)) => {
                    body.push(node);
                    continue;
                }
            }
        }
    }

    fn should_break_expression(
        &self,
        text: &str,
        break_on_infix: bool,
        break_on_token_text: Option<&str>,
    ) -> bool {
        is_end_of_expression(text)
            || (break_on_token_text.is_some_and(|stop| text == stop))
            || (break_on_infix
                && self
                    .function_registry
                    .get(text)
                    .is_some_and(|spec| spec.infix))
    }

    fn finish_expression(&mut self, body: Vec<ParseNode>) -> Result<Vec<ParseNode>, ParseError> {
        let normalized = if self.mode == Mode::Text {
            form_text_ligatures(body)
        } else {
            body
        };
        self.handle_infix_nodes(normalized)
    }

    fn consume_spaces(&mut self) -> Result<(), ParseError> {
        loop {
            if self.fetch()?.text != " " {
                break;
            }
            self.consume();
        }
        Ok(())
    }

    fn parse_atom(
        &mut self,
        break_on_token_text: Option<&str>,
    ) -> Result<Option<AtomResult>, ParseError> {
        match self.parse_group("atom", break_on_token_text)? {
            None => Ok(None),
            Some(AtomResult::SkipAtom) => Ok(Some(AtomResult::SkipAtom)),
            Some(AtomResult::EmitAtom(ParseNode::Internal { .. })) => {
                Ok(Some(AtomResult::SkipAtom))
            }
            Some(AtomResult::EmitAtom(base)) if self.mode == Mode::Text => {
                Ok(Some(AtomResult::EmitAtom(base)))
            }
            Some(AtomResult::EmitAtom(base)) => {
                Ok(Some(AtomResult::EmitAtom(self.parse_scripts(base)?)))
            }
        }
    }

    fn parse_scripts(&mut self, base: ParseNode) -> Result<ParseNode, ParseError> {
        let mut base = base;
        let mut sup: Option<ParseNode> = None;
        let mut sub: Option<ParseNode> = None;
        loop {
            self.consume_spaces()?;
            let token = self.fetch()?;
            if token.text == "\\limits" || token.text == "\\nolimits" {
                base = set_limits(base, token.text == "\\limits", token.loc.clone())?;
                self.consume();
                continue;
            } else if token.text == "^" {
                if sup.is_some() {
                    return Err(ParseError::DoubleSuperscript {
                        loc: token.loc.clone(),
                    });
                }
                sup = Some(self.handle_sup_subscript("superscript")?);
                continue;
            } else if token.text == "_" {
                if sub.is_some() {
                    return Err(ParseError::DoubleSubscript {
                        loc: token.loc.clone(),
                    });
                }
                sub = Some(self.handle_sup_subscript("subscript")?);
                continue;
            } else if token.text == "'" {
                if sup.is_some() {
                    return Err(ParseError::DoubleSuperscript {
                        loc: token.loc.clone(),
                    });
                }
                sup = Some(self.parse_prime_run()?);
                continue;
            }
            match lookup_unicode_script(&token.text) {
                None => return Ok(make_supsub_or_base(self.mode, base, sup, sub)),
                Some(first_script) => {
                    let (is_subscript, script_tokens) =
                        self.consume_unicode_script_run(first_script)?;
                    let body = self.subparse(script_tokens)?;
                    let group = ParseNode::OrdGroup {
                        mode: Mode::Math,
                        loc: None,
                        body,
                        semisimple: false,
                    };
                    if is_subscript {
                        sub = Some(group);
                    } else {
                        sup = Some(group);
                    }
                    continue;
                }
            }
        }
    }

    fn parse_prime_run(&mut self) -> Result<ParseNode, ParseError> {
        let mut primes: Vec<ParseNode> = Vec::new();
        while self.fetch()?.text == "'" {
            let prime_token = self.fetch()?;
            primes.push(ParseNode::TextOrd {
                mode: self.mode,
                loc: prime_token.loc.clone(),
                text: "\\prime".to_string(),
            });
            self.consume();
        }
        if self.fetch()?.text == "^" {
            primes.push(self.handle_sup_subscript("superscript")?);
        }
        Ok(ParseNode::OrdGroup {
            mode: self.mode,
            loc: None,
            body: primes,
            semisimple: false,
        })
    }

    fn consume_unicode_script_run(
        &mut self,
        first: &crate::unicode_scripts::UnicodeScript,
    ) -> Result<(bool, Vec<Token>), ParseError> {
        let is_subscript = first.kind == UnicodeScriptKind::UnicodeSubscript;
        let mut tokens: Vec<Token> = vec![Token::new(first.replacement.clone(), None)];
        self.consume();
        loop {
            let next = self.fetch()?;
            match lookup_unicode_script(&next.text) {
                Some(script)
                    if (script.kind == UnicodeScriptKind::UnicodeSubscript) == is_subscript =>
                {
                    tokens.push(Token::new(script.replacement.clone(), None));
                    self.consume();
                    continue;
                }
                _ => {
                    tokens.reverse();
                    return Ok((is_subscript, tokens));
                }
            }
        }
    }

    fn handle_sup_subscript(&mut self, name: &str) -> Result<ParseNode, ParseError> {
        let token = self.fetch()?;
        self.consume();
        self.consume_spaces()?;
        loop {
            match self.parse_group(name, None)? {
                Some(AtomResult::EmitAtom(ParseNode::Internal { .. }) | AtomResult::SkipAtom) => {
                    continue;
                }
                Some(AtomResult::EmitAtom(group)) => return Ok(group),
                None => {
                    return Err(ParseError::ExpectedGroupAfter {
                        symbol: token.text.clone(),
                        loc: token.loc.clone(),
                    });
                }
            }
        }
    }

    fn parse_group(
        &mut self,
        name: &str,
        break_on_token_text: Option<&str>,
    ) -> Result<Option<AtomResult>, ParseError> {
        let first_token = self.fetch()?;
        let text = first_token.text.clone();
        if text == "{" || text == "\\begingroup" {
            Ok(Some(AtomResult::EmitAtom(
                self.parse_group_body(&first_token, &text)?,
            )))
        } else {
            match self.parse_function(break_on_token_text, Some(name))? {
                Some(result) => Ok(Some(result)),
                None => match self.parse_symbol()? {
                    Some(node) => Ok(Some(AtomResult::EmitAtom(node))),
                    None => self.handle_undefined_control(&first_token),
                },
            }
        }
    }

    fn parse_group_body(
        &mut self,
        first_token: &Token,
        text: &str,
    ) -> Result<ParseNode, ParseError> {
        self.consume();
        let group_end = if text == "{" { "}" } else { "\\endgroup" };
        self.gullet.begin_group();
        let body = self.parse_expression(false, Some(group_end))?;
        let last = self.fetch()?;
        self.expect(group_end, true)?;
        self.gullet.end_group()?;
        let loc = match (&first_token.loc, &last.loc) {
            (Some(start_loc), Some(end_loc)) => Some(SourceLocation::range(start_loc, end_loc)),
            _ => None,
        };
        Ok(ParseNode::OrdGroup {
            mode: self.mode,
            loc,
            body,
            semisimple: text == "\\begingroup",
        })
    }

    fn handle_undefined_control(
        &mut self,
        token: &Token,
    ) -> Result<Option<AtomResult>, ParseError> {
        let text = token.text.clone();
        if !is_undefined_control_sequence(&text) {
            return Ok(None);
        }
        if !self.settings.throw_on_error {
            self.consume();
            return Ok(Some(AtomResult::EmitAtom(format_unsupported_command(
                self.mode,
                &self.settings.error_color,
                &text,
            ))));
        }
        Err(ParseError::UndefinedControlSequence {
            name: text,
            loc: token.loc.clone(),
        })
    }

    fn parse_function(
        &mut self,
        break_on_token_text: Option<&str>,
        name: Option<&str>,
    ) -> Result<Option<AtomResult>, ParseError> {
        let token = self.fetch()?;
        let func_data = match self.function_registry.get(&token.text) {
            None => return Ok(None),
            Some(fd) => fd.clone(),
        };
        self.consume();
        if let Some(context_name) = name {
            if context_name != "atom" && !func_data.allowed_in_argument {
                return Err(ParseError::FunctionNotAllowed {
                    func_name: token.text.clone(),
                    context: context_name.to_string(),
                    loc: token.loc.clone(),
                });
            }
        }
        if self.mode == Mode::Text && !func_data.allowed_in_text {
            return Err(ParseError::FunctionNotAllowed {
                func_name: token.text.clone(),
                context: "text mode".to_string(),
                loc: token.loc.clone(),
            });
        }
        if self.mode == Mode::Math && !func_data.allowed_in_math {
            return Err(ParseError::FunctionNotAllowed {
                func_name: token.text.clone(),
                context: "math mode".to_string(),
                loc: token.loc.clone(),
            });
        }
        let (args, opt_args) = self.parse_arguments(&token.text, &func_data)?;
        Ok(Some(AtomResult::EmitAtom(self.call_function(
            &token.text,
            args,
            opt_args,
            Some(token.clone()),
            break_on_token_text.map(ToString::to_string),
        )?)))
    }

    fn call_function(
        &mut self,
        func_name: &str,
        args: Vec<ParseNode>,
        opt_args: Vec<Option<ParseNode>>,
        token: Option<Token>,
        break_on_token_text: Option<String>,
    ) -> Result<ParseNode, ParseError> {
        let context = FunctionContext {
            func_name: func_name.to_string(),
            mode: self.mode,
            token: token.clone(),
            break_on_token_text: break_on_token_text.clone(),
            display_mode: self.settings.display_mode,
        };
        let handler = {
            let spec = self.function_registry.get(func_name).ok_or_else(|| {
                ParseError::MissingFunctionHandler {
                    func_name: func_name.to_string(),
                    loc: None,
                }
            })?;
            spec.handler
                .ok_or_else(|| ParseError::MissingFunctionHandler {
                    func_name: func_name.to_string(),
                    loc: None,
                })?
        };
        handler(self, &context, &args, &opt_args)
    }

    fn parse_arguments(
        &mut self,
        func: &str,
        func_data: &FunctionSpec,
    ) -> Result<(Vec<ParseNode>, Vec<Option<ParseNode>>), ParseError> {
        let mut args: Vec<ParseNode> = Vec::new();
        let mut opt_args: Vec<Option<ParseNode>> = Vec::new();
        let total_args = func_data.num_args + func_data.num_optional_args;
        let mut index = 0;
        while index < total_args {
            let optional = index < func_data.num_optional_args;
            let arg_type = match func_data.arg_types.get(index) {
                Some(kind) => *kind,
                None if func_data.primitive => ArgType::PrimitiveArg,
                None if func_data.primitive_after_missing_optional.is_some_and(
                    |optional_index| {
                        index == func_data.num_optional_args
                            && opt_args.get(optional_index).is_some_and(Option::is_none)
                    },
                ) =>
                {
                    ArgType::PrimitiveArg
                }
                None => ArgType::OriginalArg,
            };
            match self.parse_group_of_type(&format!("argument to '{func}'"), arg_type, optional)? {
                None if optional => {
                    opt_args.push(None);
                    index += 1;
                    continue;
                }
                None => {
                    return Err(ParseError::InternalInvariant {
                        message: "Null mandatory function argument after parser validation"
                            .to_string(),
                    });
                }
                Some(arg) if optional => {
                    opt_args.push(Some(arg));
                    index += 1;
                    continue;
                }
                Some(arg) => {
                    args.push(arg);
                    index += 1;
                    continue;
                }
            }
        }
        Ok((args, opt_args))
    }

    fn parse_group_of_type(
        &mut self,
        name: &str,
        arg_type: ArgType,
        optional: bool,
    ) -> Result<Option<ParseNode>, ParseError> {
        match arg_type {
            ArgType::ColorArg => self.parse_color_group(optional),
            ArgType::SizeArg => self.parse_size_group(optional),
            ArgType::UrlArg => self.parse_url_group(optional),
            ArgType::RawArg => Ok(self
                .parse_string_group(optional)?
                .map(|token| ParseNode::Raw {
                    mode: Mode::Text,
                    string: token.text,
                })),
            ArgType::MathArg => self.parse_argument_group(optional, Some(Mode::Math)),
            ArgType::TextArg => self.parse_argument_group(optional, Some(Mode::Text)),
            ArgType::HboxArg => {
                Ok(self
                    .parse_argument_group(optional, Some(Mode::Text))?
                    .map(|group| ParseNode::Styling {
                        mode: group.mode(),
                        body: vec![group],
                        style: StyleLevel::TextStyle,
                        reset_font: true,
                    }))
            }
            ArgType::PrimitiveArg => self.parse_primitive_group(name, optional),
            ArgType::OriginalArg => self.parse_argument_group(optional, None),
        }
    }

    fn parse_primitive_group(
        &mut self,
        name: &str,
        optional: bool,
    ) -> Result<Option<ParseNode>, ParseError> {
        if optional {
            return Err(ParseError::InvalidArgument {
                message: "A primitive argument cannot be optional".to_string(),
                loc: None,
            });
        }
        let Some(AtomResult::EmitAtom(group)) = self.parse_group(name, None)? else {
            let token = self.fetch()?;
            return Err(ParseError::InvalidArgument {
                message: format!("Expected group as {name}"),
                loc: token.loc,
            });
        };
        Ok(Some(group))
    }

    fn parse_argument_group(
        &mut self,
        optional: bool,
        mode: Option<Mode>,
    ) -> Result<Option<ParseNode>, ParseError> {
        match self.gullet.scan_argument(optional)? {
            None => Ok(None),
            Some(arg_token) => {
                let outer_mode = self.mode;
                if let Some(argument_mode) = mode {
                    self.switch_mode(argument_mode);
                }
                self.gullet.begin_group();
                let result: Result<ParseNode, ParseError> = (|| {
                    let body = self.parse_expression(false, Some("EOF"))?;
                    self.expect("EOF", true)?;
                    Ok(ParseNode::OrdGroup {
                        mode: self.mode,
                        loc: arg_token.loc.clone(),
                        body,
                        semisimple: false,
                    })
                })();
                let close_result = self.gullet.end_group();
                self.switch_mode(outer_mode);
                Ok(Some(unwrap_captured(close_result, result)?))
            }
        }
    }

    fn parse_string_group(&mut self, optional: bool) -> Result<Option<Token>, ParseError> {
        match self.gullet.scan_argument(optional)? {
            None => Ok(None),
            Some(mut arg_token) => {
                let mut builder = String::new();
                loop {
                    let token = self.fetch()?;
                    if token.text == "EOF" {
                        self.consume();
                        arg_token.text = builder;
                        break Ok(Some(arg_token));
                    }
                    builder.push_str(&token.text);
                    self.consume();
                }
            }
        }
    }

    fn parse_color_group(&mut self, optional: bool) -> Result<Option<ParseNode>, ParseError> {
        match self.parse_string_group(optional)? {
            None => Ok(None),
            Some(token) => {
                let color =
                    normalized_color(&token.text).ok_or_else(|| ParseError::InvalidArgument {
                        message: format!("Invalid color: '{}'", token.text),
                        loc: token.loc.clone(),
                    })?;
                Ok(Some(ParseNode::ColorToken {
                    mode: self.mode,
                    color,
                }))
            }
        }
    }

    fn parse_url_group(&mut self, optional: bool) -> Result<Option<ParseNode>, ParseError> {
        self.gullet.set_lexer_catcode("%", 13);
        self.gullet.set_lexer_catcode("~", 12);
        let parsed = self.parse_string_group(optional);
        self.gullet.set_lexer_catcode("%", 14);
        self.gullet.set_lexer_catcode("~", 13);
        match parsed? {
            None => Ok(None),
            Some(token) => Ok(Some(ParseNode::Url {
                mode: self.mode,
                url: unescape_url(&token.text),
            })),
        }
    }

    fn parse_size_regex_group(&mut self) -> Result<Token, ParseError> {
        let first_token = self.fetch()?;
        let mut last_token = first_token.clone();
        let mut builder = String::new();
        loop {
            let token = self.fetch()?;
            if token.text == "EOF" {
                break;
            }
            let candidate = format!("{builder}{}", token.text);
            if !size_scan_candidate(&candidate) {
                break;
            }
            builder.push_str(&token.text);
            last_token = token;
            self.consume();
        }
        if builder.is_empty() {
            return Err(ParseError::InvalidArgument {
                message: format!("Invalid size: '{}'", first_token.text),
                loc: first_token.loc.clone(),
            });
        }
        Ok(first_token.range(&last_token, builder))
    }

    fn parse_size_group(&mut self, optional: bool) -> Result<Option<ParseNode>, ParseError> {
        self.gullet.consume_spaces()?;
        let parsed = if !optional && self.gullet.future()?.text != "{" {
            Some(self.parse_size_regex_group()?)
        } else {
            self.parse_string_group(optional)?
        };
        match parsed {
            None => Ok(None),
            Some(token) => {
                let mut text = token.text;
                let is_blank = !optional && text.is_empty();
                if is_blank {
                    text = "0pt".to_string();
                }
                let value =
                    parse_size_measurement(&text)?.ok_or_else(|| ParseError::InvalidArgument {
                        message: format!("Invalid size: '{text}'"),
                        loc: token.loc.clone(),
                    })?;
                if !valid_size_unit(&value.unit) {
                    return Err(ParseError::InvalidArgument {
                        message: format!("Invalid unit: '{}'", value.unit),
                        loc: token.loc.clone(),
                    });
                }
                Ok(Some(ParseNode::Size {
                    mode: self.mode,
                    value,
                    is_blank,
                }))
            }
        }
    }

    fn handle_infix_nodes(&mut self, body: Vec<ParseNode>) -> Result<Vec<ParseNode>, ParseError> {
        let mut infix: Option<(usize, String)> = None;
        for (index, node) in body.iter().enumerate() {
            if let ParseNode::Infix {
                replace_with, loc, ..
            } = node
            {
                if infix.is_some() {
                    return Err(ParseError::InvalidArgument {
                        message: "only one infix operator per group".to_string(),
                        loc: loc.clone(),
                    });
                }
                infix = Some((index, replace_with.clone()));
            }
        }
        if let Some((index, func_name)) = infix {
            let numer = infix_side_group(self.mode, body[..index].to_vec());
            let denom = infix_side_group(self.mode, body[index + 1..].to_vec());
            let node = if func_name == "\\\\abovefrac" {
                self.call_function(
                    &func_name,
                    vec![numer, body[index].clone(), denom],
                    Vec::new(),
                    None,
                    None,
                )?
            } else {
                self.call_function(&func_name, vec![numer, denom], Vec::new(), None, None)?
            };
            Ok(vec![node])
        } else {
            Ok(body)
        }
    }

    fn parse_environment(&mut self, name: &str) -> Result<ParseNode, ParseError> {
        let spec = self
            .environment_registry
            .get(name)
            .cloned()
            .ok_or_else(|| ParseError::InvalidArgument {
                message: format!("No such environment: {name}"),
                loc: None,
            })?;
        let arguments = FunctionSpec {
            names: Vec::new(),
            num_args: spec.num_args,
            num_optional_args: spec.num_optional_args,
            arg_types: spec.arg_types.clone(),
            ..Default::default()
        };
        let (args, opt_args) = self.parse_arguments(&format!("\\begin{{{name}}}"), &arguments)?;
        let context = EnvironmentContext {
            mode: self.mode,
            display_mode: self.settings.display_mode,
            leqno: self.settings.leqno,
            env_name: name.to_string(),
        };
        let result = (spec.handler)(self, &context, &args, &opt_args)?;
        self.expect("\\end", false)?;
        match self.parse_function(None, None)? {
            Some(AtomResult::EmitAtom(ParseNode::EnvironmentEnd { name: end_name, .. }))
                if end_name == name =>
            {
                Ok(result)
            }
            Some(AtomResult::EmitAtom(ParseNode::EnvironmentEnd { name: end_name, .. })) => {
                Err(ParseError::InvalidArgument {
                    message: format!("Mismatch: \\begin{{{name}}} matched by \\end{{{end_name}}}"),
                    loc: None,
                })
            }
            _ => Err(ParseError::InternalInvariant {
                message: "Expected environment end".to_string(),
            }),
        }
    }

    fn parse_matrix_alignment(&mut self) -> Result<Option<String>, ParseError> {
        self.consume_spaces()?;
        if self.fetch()?.text != "[" {
            return Ok(None);
        }
        self.consume();
        self.consume_spaces()?;
        let token = self.fetch()?;
        if token.text != "l" && token.text != "c" && token.text != "r" {
            return Err(ParseError::InvalidArgument {
                message: "Expected l or c or r".to_string(),
                loc: token.loc.clone(),
            });
        }
        self.consume();
        self.consume_spaces()?;
        self.expect("]", true)?;
        Ok(Some(token.text))
    }

    fn parse_prefixed_function(&mut self, name: &str) -> Result<ParseNode, ParseError> {
        self.gullet.push_token(Token::new(name, None));
        let Some(AtomResult::EmitAtom(node)) = self.parse_function(None, None)? else {
            return Err(ParseError::InternalInvariant {
                message: "Expected function after macro prefix".to_string(),
            });
        };
        Ok(node)
    }

    fn parse_symbol(&mut self) -> Result<Option<ParseNode>, ParseError> {
        let token = self.fetch()?;
        let original_text = token.text.clone();
        if original_text == "EOF"
            || original_text == "^"
            || original_text == "_"
            || original_text == "{"
            || original_text == "}"
            || original_text == "&"
        {
            Ok(None)
        } else if is_verb_token(&original_text) {
            self.consume();
            Ok(Some(parse_verb_token(&original_text)?))
        } else {
            self.parse_symbol_text(
                &token,
                &original_text,
                normalize_unicode_symbol(self.mode, &original_text),
            )
        }
    }

    fn parse_symbol_text(
        &mut self,
        token: &Token,
        original_text: &str,
        normalized: String,
    ) -> Result<Option<ParseNode>, ParseError> {
        if self.mode == Mode::Math && normalized != original_text {
            let first = original_text.chars().next().unwrap();
            self.settings.report_nonstrict(
                "unicodeTextInMathMode",
                &format!("Accented Unicode text character \"{first}\" used in math mode"),
                Some(token),
            )?;
        }
        let (text, marks) = split_combining_marks(&normalized);
        match lookup_symbol(self.mode, &text) {
            Some(spec) => {
                if self.mode == Mode::Math && is_extra_latin(&text) {
                    let first = text.chars().next().unwrap();
                    self.settings.report_nonstrict(
                        "unicodeTextInMathMode",
                        &format!("Latin-1/Unicode text character \"{first}\" used in math mode"),
                        Some(token),
                    )?;
                }
                self.consume();
                let base = make_symbol_node(self.mode, &text, token.loc.clone(), spec);
                match marks {
                    None => Ok(Some(base)),
                    Some(accents) => Ok(Some(apply_unicode_accents(
                        self.mode,
                        token.loc.clone(),
                        base,
                        &accents,
                    )?)),
                }
            }
            None if is_non_ascii(&text) => {
                let first = text.chars().next().unwrap();
                if !supported_codepoint(first as u32) {
                    self.settings.report_nonstrict(
                        "unknownSymbol",
                        &format!(
                            "Unrecognized Unicode character \"{first}\" ({})",
                            first as u32
                        ),
                        Some(token),
                    )?;
                } else if self.mode == Mode::Math {
                    self.settings.report_nonstrict(
                        "unicodeTextInMathMode",
                        &format!("Unicode text character \"{first}\" used in math mode"),
                        Some(token),
                    )?;
                }
                self.consume();
                Ok(Some(ParseNode::TextOrd {
                    mode: Mode::Text,
                    loc: token.loc.clone(),
                    text,
                }))
            }
            None => Ok(None),
        }
    }

    fn switch_mode(&mut self, mode: Mode) {
        if self.mode != mode {
            self.mode = mode;
            self.gullet.switch_mode(mode);
        }
    }

    fn array_stretch(&mut self, explicit: Option<f64>) -> Result<f64, ParseError> {
        if let Some(stretch) = explicit {
            return Ok(stretch);
        }
        let Some(text) = self.gullet.expand_macro_as_text("\\arraystretch")? else {
            return Ok(1.0);
        };
        parse_array_stretch_number(&text)
            .filter(|stretch| *stretch > 0.0)
            .ok_or_else(|| ParseError::InvalidArgument {
                message: format!("Invalid \\arraystretch: {text}"),
                loc: None,
            })
    }

    fn begin_array_row(&mut self, auto_tag: Option<bool>) {
        if auto_tag == Some(true) {
            self.gullet.macros.set(
                "\\@eqnsw".to_string(),
                Some(MacroDefinition::text("1")),
                true,
            );
        }
    }

    fn consume_array_hlines(&mut self) -> Result<Vec<bool>, ParseError> {
        let mut lines: Vec<bool> = Vec::new();
        self.consume_spaces()?;
        if self.fetch()?.text == "\\relax" {
            self.consume();
            self.consume_spaces()?;
        }
        while self.fetch()?.text == "\\hline" || self.fetch()?.text == "\\hdashline" {
            let dashed = self.fetch()?.text == "\\hdashline";
            self.consume();
            lines.push(dashed);
            self.consume_spaces()?;
        }
        Ok(lines)
    }

    fn take_array_tag(
        &mut self,
        auto_tag: Option<bool>,
    ) -> Result<(Option<Vec<ParseNode>>, bool), ParseError> {
        let Some(automatic) = auto_tag else {
            return Ok((None, false));
        };
        if self.gullet.macros.get("\\df@tag").is_none() {
            let enabled = matches!(
                self.gullet.macros.get("\\@eqnsw"),
                Some(MacroDefinition::Text(text)) if text == "1"
            );
            return Ok((None, automatic && enabled));
        }
        let tag = self.subparse(vec![Token::new("\\df@tag", None)])?;
        self.gullet.macros.set("\\df@tag".to_string(), None, true);
        Ok((Some(tag), false))
    }

    fn push_array_tag(
        &mut self,
        tags: &mut Vec<Option<Vec<ParseNode>>>,
        auto_tags: &mut Vec<bool>,
        auto_tag: Option<bool>,
    ) -> Result<(), ParseError> {
        let (tag, automatic) = self.take_array_tag(auto_tag)?;
        if auto_tag.is_some() {
            tags.push(tag);
            auto_tags.push(automatic);
        }
        Ok(())
    }

    fn parse_array_row_gap(&mut self) -> Result<Option<Measurement>, ParseError> {
        if self.gullet.future()?.text == " " {
            Ok(None)
        } else {
            match self.parse_size_group(true)? {
                Some(ParseNode::Size { value, .. }) => Ok(Some(value)),
                Some(_) => Err(ParseError::InternalInvariant {
                    message: "Expected array row gap".to_string(),
                }),
                None => Ok(None),
            }
        }
    }

    fn parse_array_environment(
        &mut self,
        options: ArrayEnvironmentOptions,
    ) -> Result<ParseNode, ParseError> {
        self.gullet.begin_group();
        if !options.single_row {
            self.gullet.macros.set(
                "\\cr".to_string(),
                Some(MacroDefinition::text("\\\\\\relax")),
                false,
            );
        }
        self.gullet.begin_group();
        let result: Result<ParseNode, ParseError> = (|| {
            let array_stretch = self.array_stretch(options.array_stretch)?;
            self.begin_array_row(options.auto_tag);
            let mut body: Vec<Vec<ParseNode>> = vec![Vec::new()];
            let mut row_gaps: Vec<Option<Measurement>> = Vec::new();
            let mut hlines_before_row: Vec<Vec<bool>> = vec![self.consume_array_hlines()?];
            let mut tags: Vec<Option<Vec<ParseNode>>> = Vec::new();
            let mut auto_tags: Vec<bool> = Vec::new();
            loop {
                let cell_body = self.parse_expression(false, Some("\\\\"))?;
                let empty_cell = cell_body.is_empty();
                let cell = ParseNode::Styling {
                    mode: self.mode,
                    body: vec![ParseNode::OrdGroup {
                        mode: self.mode,
                        loc: None,
                        body: cell_body,
                        semisimple: false,
                    }],
                    style: options.cell_style,
                    reset_font: true,
                };
                self.gullet.end_group()?;
                self.gullet.begin_group();
                let Some(row) = body.last_mut() else {
                    return Err(ParseError::InternalInvariant {
                        message: "Missing array row".to_string(),
                    });
                };
                row.push(cell);
                let text = self.fetch()?.text;
                match text.as_str() {
                    "&" => {
                        if array_row_at_max(&body, options.max_columns) {
                            return Err(ParseError::InvalidArgument {
                                message: "Too many tab characters: &".to_string(),
                                loc: None,
                            });
                        }
                        self.consume();
                    }
                    "\\end" => {
                        self.push_array_tag(&mut tags, &mut auto_tags, options.auto_tag)?;
                        if row.len() == 1
                            && empty_cell
                            && (body.len() > 1 || !options.empty_single_row)
                            && !tags.last().is_some_and(Option::is_some)
                            && auto_tags.last() != Some(&true)
                        {
                            body.pop();
                        }
                        break;
                    }
                    "\\\\" => {
                        if options.single_row {
                            return Err(ParseError::InvalidArgument {
                                message: "Expected \\end".to_string(),
                                loc: None,
                            });
                        }
                        self.consume();
                        row_gaps.push(self.parse_array_row_gap()?);
                        self.push_array_tag(&mut tags, &mut auto_tags, options.auto_tag)?;
                        hlines_before_row.push(self.consume_array_hlines()?);
                        body.push(Vec::new());
                        self.begin_array_row(options.auto_tag);
                    }
                    _ => {
                        return Err(ParseError::InvalidArgument {
                            message: format!("Expected & or \\\\ or \\end, got {text}"),
                            loc: None,
                        });
                    }
                }
            }
            if hlines_before_row.len() < body.len() + 1 {
                hlines_before_row.push(Vec::new());
            }
            Ok(ParseNode::Array {
                mode: self.mode,
                body,
                add_jot: options.add_jot,
                array_stretch,
                columns: options.columns.clone(),
                row_gaps,
                hskip_before_and_after: options.hskip_before_and_after,
                hlines_before_row,
                column_separation_type: options.column_separation_type,
                tags: if options.auto_tag.is_some() {
                    Some(tags)
                } else {
                    None
                },
                auto_tags: if options.auto_tag.is_some() {
                    Some(auto_tags)
                } else {
                    None
                },
                leqno: options.leqno,
            })
        })();
        let close_cell = self.gullet.end_group();
        let close_array = self.gullet.end_group();
        unwrap_array_parse_result(result, close_cell, close_array)
    }

    fn parse_cd_environment(&mut self) -> Result<ParseNode, ParseError> {
        self.gullet.begin_group();
        self.gullet.macros.set(
            "\\cr".to_string(),
            Some(MacroDefinition::text("\\\\\\relax")),
            false,
        );
        self.gullet.begin_group();
        let result: Result<ParseNode, ParseError> = (|| {
            let mut parsed_rows: Vec<Vec<ParseNode>> = Vec::new();
            loop {
                parsed_rows.push(self.parse_expression(false, Some("\\\\"))?);
                self.gullet.end_group()?;
                self.gullet.begin_group();
                match self.fetch()?.text.as_str() {
                    "&" | "\\\\" => self.consume(),
                    "\\end" => {
                        if parsed_rows.last().is_some_and(Vec::is_empty) {
                            parsed_rows.pop();
                        }
                        break;
                    }
                    token => {
                        return Err(ParseError::InvalidArgument {
                            message: format!("Expected \\ or \\cr or \\end, got {token}"),
                            loc: None,
                        });
                    }
                }
            }
            let mut body: Vec<Vec<ParseNode>> = Vec::new();
            for (index, row) in parsed_rows.into_iter().enumerate() {
                body.push(cd_row(row, index % 2 == 0)?);
            }
            // KaTeX parseCD keeps a final structural empty row.
            body.push(Vec::new());
            let count = body.first().map_or(0, Vec::len);
            let columns: Vec<crate::ast::ArrayColumn> = (0..count)
                .map(|_| crate::ast::ArrayColumn::AlignColumn {
                    alignment: "c".to_string(),
                    pre_gap: Some(0.25),
                    post_gap: Some(0.25),
                })
                .collect();
            let row_gap_count = body.len() + 1;
            let hlines_before_row: Vec<Vec<bool>> =
                (0..row_gap_count).map(|_| Vec::new()).collect();
            Ok(ParseNode::Array {
                mode: Mode::Math,
                body,
                add_jot: true,
                array_stretch: 1.0,
                columns: Some(columns),
                row_gaps: vec![None],
                hskip_before_and_after: false,
                hlines_before_row,
                column_separation_type: Some(ColumnSeparationType::CdSeparation),
                tags: None,
                auto_tags: None,
                leqno: false,
            })
        })();
        let close_cell = self.gullet.end_group();
        let close_array = self.gullet.end_group();
        unwrap_array_parse_result(result, close_cell, close_array)
    }
}

impl FunctionParser for Parser {
    fn report_nonstrict(
        &self,
        error_code: &str,
        error_message: &str,
        token: Option<&Token>,
    ) -> Result<(), ParseError> {
        self.settings
            .report_nonstrict(error_code, error_message, token)
    }

    fn use_strict_behavior(
        &self,
        error_code: &str,
        error_message: &str,
        token: Option<&Token>,
    ) -> bool {
        self.settings
            .use_strict_behavior(error_code, error_message, token)
    }

    fn is_trusted(&self, context: TrustContext) -> bool {
        self.settings.is_trusted(context)
    }

    fn unsupported_command(&self, mode: Mode, text: &str, fallback: ParseNode) -> ParseNode {
        unsupported_command(mode, &self.settings.error_color, text, fallback)
    }

    fn current_color(&self) -> Result<Option<String>, ParseError> {
        self.current_color()
    }

    fn in_left_right(&self) -> bool {
        self.leftright_depth > 0
    }

    fn is_expandable(&self, name: &str) -> bool {
        self.gullet.is_expandable(name)
    }

    fn get_macro(&self, name: &str) -> Option<MacroDefinition> {
        self.gullet.macros.get(name).cloned()
    }

    fn set_macro(&mut self, name: &str, definition: Option<MacroDefinition>) {
        self.gullet.macros.set(name.to_string(), definition, false);
    }

    fn set_macro_definition(&mut self, name: &str, definition: MacroDefinition, global: bool) {
        self.gullet
            .macros
            .set(name.to_string(), Some(definition), global);
    }

    fn parse_expression(
        &mut self,
        break_on_infix: bool,
        break_on_token_text: Option<&str>,
    ) -> Result<Vec<ParseNode>, ParseError> {
        self.parse_expression(break_on_infix, break_on_token_text)
    }

    fn parse_math_mode(&mut self, closing: &str) -> Result<Vec<ParseNode>, ParseError> {
        self.parse_math_mode(closing)
    }

    fn parse_left_right(&mut self, open: &str) -> Result<ParseNode, ParseError> {
        self.parse_left_right(open)
    }

    fn parse_optional_size(&mut self) -> Result<Option<Measurement>, ParseError> {
        if self.gullet.future()?.text != "[" {
            Ok(None)
        } else {
            match self.parse_size_group(true)? {
                Some(ParseNode::Size { value, .. }) => Ok(Some(value)),
                _ => Err(ParseError::InternalInvariant {
                    message: "Expected optional size".to_string(),
                }),
            }
        }
    }

    fn parse_prefixed_function(&mut self, name: &str) -> Result<ParseNode, ParseError> {
        self.parse_prefixed_function(name)
    }

    fn parse_environment(&mut self, name: &str) -> Result<ParseNode, ParseError> {
        self.parse_environment(name)
    }

    fn pop_token(&mut self) -> Result<Token, ParseError> {
        self.gullet.pop_token()
    }

    fn future_token(&mut self) -> Result<Token, ParseError> {
        self.gullet.future()
    }

    fn push_token(&mut self, token: Token) {
        self.gullet.push_token(token);
    }

    fn consume_spaces(&mut self) -> Result<(), ParseError> {
        self.gullet.consume_spaces()
    }

    fn consume_macro_arg(&mut self) -> Result<Vec<Token>, ParseError> {
        Ok(self.gullet.consume_arg(None)?.tokens)
    }

    fn expand_tokens(&mut self, tokens: Vec<Token>) -> Result<Vec<Token>, ParseError> {
        self.gullet.expand_tokens(tokens)
    }
}

impl EnvironmentParser for Parser {
    fn parse_array(&mut self, options: ArrayEnvironmentOptions) -> Result<ParseNode, ParseError> {
        self.parse_array_environment(options)
    }

    fn parse_matrix_alignment(&mut self) -> Result<Option<String>, ParseError> {
        self.parse_matrix_alignment()
    }

    fn parse_cd(&mut self) -> Result<ParseNode, ParseError> {
        self.parse_cd_environment()
    }
}

/// Parses `input` into a `ParseNode` list using the given settings. When
/// `settings.global_group` is set and `settings.macro_store` is present,
/// macros defined during the parse (e.g. via `\newcommand`) are written back
/// into the caller's store.
pub fn parse(input: &str, settings: &mut Settings) -> Result<Vec<ParseNode>, ParseError> {
    parse_with_specs(input, settings, &[], &[])
}

/// Parses `input` with caller-provided extension function/environment specs.
pub fn parse_with_specs(
    input: &str,
    settings: &mut Settings,
    extra_specs: &[FunctionSpec],
    extra_env_specs: &[EnvironmentSpec],
) -> Result<Vec<ParseNode>, ParseError> {
    let mut parser = Parser::new(input, settings.clone(), extra_specs, extra_env_specs);
    parser
        .gullet
        .macros
        .set("\\df@tag".to_string(), None, false);
    let parse_result = parser.parse();
    if settings.global_group {
        settings.macro_store = parser.settings.macro_store.clone();
    }
    let mut body = parse_result?;
    if parser.gullet.macros.get("\\df@tag").is_some() {
        if !settings.display_mode {
            return Err(ParseError::InvalidArgument {
                message: "\\tag works only in display equations".to_string(),
                loc: None,
            });
        }
        let tag = parser.subparse(vec![Token::new("\\df@tag", None)])?;
        body = vec![ParseNode::Tag {
            mode: Mode::Text,
            body,
            tag,
        }];
    }
    parser
        .gullet
        .macros
        .set("\\current@color".to_string(), None, false);
    parser.gullet.macros.set("\\color".to_string(), None, false);
    if settings.display_mode {
        Ok(vec![ParseNode::Styling {
            mode: Mode::Math,
            body,
            style: StyleLevel::DisplayStyle,
            reset_font: true,
        }])
    } else {
        Ok(body)
    }
}

fn unwrap_captured<V>(
    close: Result<(), ParseError>,
    value: Result<V, ParseError>,
) -> Result<V, ParseError> {
    match (close, value) {
        (Err(err), _) => Err(err),
        (_, Err(err)) => Err(err),
        (Ok(()), Ok(value)) => Ok(value),
    }
}

fn is_end_of_expression(text: &str) -> bool {
    text == "}" || text == "\\endgroup" || text == "\\end" || text == "\\right" || text == "&"
}

fn set_limits(
    base: ParseNode,
    limits: bool,
    loc: Option<SourceLocation>,
) -> Result<ParseNode, ParseError> {
    match base {
        ParseNode::Op {
            mode,
            parent_is_sup_sub,
            suppress_base_shift,
            content,
            ..
        } => Ok(ParseNode::Op {
            mode,
            limits,
            always_handle_sup_sub: true,
            parent_is_sup_sub,
            suppress_base_shift,
            content,
        }),
        ParseNode::OperatorName {
            mode,
            body,
            always_handle_sup_sub: true,
            parent_is_sup_sub,
            ..
        } => Ok(ParseNode::OperatorName {
            mode,
            body,
            always_handle_sup_sub: true,
            limits,
            parent_is_sup_sub,
        }),
        _ => Err(ParseError::InvalidArgument {
            message: "Limit controls must follow a math operator".to_string(),
            loc,
        }),
    }
}

fn make_supsub_or_base(
    mode: Mode,
    base: ParseNode,
    sup: Option<ParseNode>,
    sub: Option<ParseNode>,
) -> ParseNode {
    match (sup, sub) {
        (None, None) => base,
        (sup, sub) => ParseNode::SupSub {
            mode,
            base: Some(Box::new(base)),
            sup: sup.map(Box::new),
            sub: sub.map(Box::new),
        },
    }
}

fn infix_side_group(mode: Mode, body: Vec<ParseNode>) -> ParseNode {
    if body.len() == 1 && matches!(body[0], ParseNode::OrdGroup { .. }) {
        body[0].clone()
    } else {
        ParseNode::OrdGroup {
            mode,
            loc: None,
            body,
            semisimple: false,
        }
    }
}

fn is_undefined_control_sequence(text: &str) -> bool {
    !text.is_empty() && text.starts_with('\\') && !is_implicit_command(text)
}

fn is_verb_token(text: &str) -> bool {
    let chars: Vec<char> = text.chars().collect();
    chars.len() > 5 && starts_with_at(&chars, 0, "\\verb") && !is_ascii_alphabetic(chars[5])
}

fn parse_verb_token(text: &str) -> Result<ParseNode, ParseError> {
    let chars: Vec<char> = text.chars().collect();
    let raw_argument = &chars[5..];
    let star = !raw_argument.is_empty() && raw_argument[0] == '*';
    let argument: &[char] = if star {
        &raw_argument[1..]
    } else {
        raw_argument
    };
    if argument.len() < 2 || argument[0] != argument[argument.len() - 1] {
        Err(ParseError::InternalInvariant {
            message: "\\verb assertion failed -- please report what input caused this bug"
                .to_string(),
        })
    } else {
        Ok(ParseNode::Verb {
            mode: Mode::Text,
            loc: None,
            body: argument[1..argument.len() - 1].iter().collect(),
            star,
        })
    }
}

fn is_extra_latin(text: &str) -> bool {
    !text.is_empty() && matches!(text.chars().next(), Some('Ð' | 'Þ' | 'þ'))
}

fn is_non_ascii(text: &str) -> bool {
    !text.is_empty() && text.chars().next().is_some_and(|c| c as u32 >= 0x80)
}

fn split_combining_marks(normalized: &str) -> (String, Option<String>) {
    match trailing_combining_mark_start(normalized) {
        None => (normalized.to_string(), None),
        Some(start) => {
            let chars: Vec<char> = normalized.chars().collect();
            let base: String = chars[..start].iter().collect();
            let base = if base == "i" {
                "ı".to_string()
            } else if base == "j" {
                "ȷ".to_string()
            } else {
                base
            };
            let marks: String = chars[start..].iter().collect();
            (base, Some(marks))
        }
    }
}

fn apply_unicode_accents(
    mode: Mode,
    loc: Option<SourceLocation>,
    base: ParseNode,
    accents: &str,
) -> Result<ParseNode, ParseError> {
    let mut result = base;
    for accent in accents.chars() {
        let accent_text = accent.to_string();
        let Some(label) = unicode_accent_command(mode, &accent_text) else {
            return Err(ParseError::InvalidArgument {
                message: format!("Unknown accent ' {accent_text}'"),
                loc: loc.clone(),
            });
        };
        result = ParseNode::Accent {
            mode,
            loc: loc.clone(),
            label,
            is_stretchy: false,
            is_shifty: true,
            base: Box::new(result),
        };
    }
    Ok(result)
}

fn make_symbol_node(
    mode: Mode,
    text: &str,
    loc: Option<SourceLocation>,
    spec: &SymbolSpec,
) -> ParseNode {
    let text = text.to_string();
    match spec.group {
        SymbolGroup::AccentTokenGroup => ParseNode::AccentToken { mode, loc, text },
        SymbolGroup::BinaryGroup => ParseNode::Atom {
            mode,
            loc,
            family: AtomFamily::Mbin,
            text,
        },
        SymbolGroup::CloseGroup => ParseNode::Atom {
            mode,
            loc,
            family: AtomFamily::Mclose,
            text,
        },
        SymbolGroup::InnerGroup => ParseNode::Atom {
            mode,
            loc,
            family: AtomFamily::Minner,
            text,
        },
        SymbolGroup::MathOrdGroup => ParseNode::MathOrd { mode, loc, text },
        SymbolGroup::OperatorTokenGroup => ParseNode::OperatorToken { mode, loc, text },
        SymbolGroup::OpenGroup => ParseNode::Atom {
            mode,
            loc,
            family: AtomFamily::Mopen,
            text,
        },
        SymbolGroup::PunctuationGroup => ParseNode::Atom {
            mode,
            loc,
            family: AtomFamily::Mpunct,
            text,
        },
        SymbolGroup::RelationGroup => ParseNode::Atom {
            mode,
            loc,
            family: AtomFamily::Mrel,
            text,
        },
        SymbolGroup::SpacingGroup => ParseNode::Spacing { mode, loc, text },
        SymbolGroup::TextOrdGroup => ParseNode::TextOrd { mode, loc, text },
    }
}

pub(crate) fn format_unsupported_command(mode: Mode, error_color: &str, text: &str) -> ParseNode {
    let body: Vec<ParseNode> = text
        .chars()
        .map(|ch| ParseNode::TextOrd {
            mode: Mode::Text,
            loc: None,
            text: ch.to_string(),
        })
        .collect();
    ParseNode::Color {
        mode,
        color: error_color.to_string(),
        body: vec![ParseNode::Text {
            mode,
            font: String::new(),
            body,
        }],
    }
}

pub(crate) fn unsupported_command(
    mode: Mode,
    color: &str,
    command: &str,
    fallback: ParseNode,
) -> ParseNode {
    ParseNode::Unsupported {
        mode,
        command: command.to_string(),
        color: color.to_string(),
        fallback: Box::new(fallback),
    }
}

/// Reads the longest decimal prefix accepted by JavaScript's parseFloat.
fn parse_array_stretch_number(text: &str) -> Option<f64> {
    let text = text.trim_start_matches(|ch| {
        matches!(ch,
            '\u{0009}'..='\u{000d}' | ' ' | '\u{00a0}' | '\u{1680}' |
            '\u{2000}'..='\u{200a}' | '\u{2028}' | '\u{2029}' | '\u{202f}' |
            '\u{205f}' | '\u{3000}' | '\u{feff}'
        )
    });
    let bytes = text.as_bytes();
    let mut index = usize::from(matches!(bytes.first(), Some(b'+' | b'-')));
    if text[index..].starts_with("Infinity") {
        return Some(if bytes.first() == Some(&b'-') {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        });
    }
    let integer_start = index;
    while bytes.get(index).is_some_and(u8::is_ascii_digit) {
        index += 1;
    }
    let integer_digits = index - integer_start;
    let mut fraction_digits = 0;
    if bytes.get(index) == Some(&b'.') {
        index += 1;
        let fraction_start = index;
        while bytes.get(index).is_some_and(u8::is_ascii_digit) {
            index += 1;
        }
        fraction_digits = index - fraction_start;
    }
    if integer_digits == 0 && fraction_digits == 0 {
        return None;
    }
    if matches!(bytes.get(index), Some(b'e' | b'E')) {
        let exponent_start = index;
        index += 1;
        if matches!(bytes.get(index), Some(b'+' | b'-')) {
            index += 1;
        }
        let digits_start = index;
        while bytes.get(index).is_some_and(u8::is_ascii_digit) {
            index += 1;
        }
        if index == digits_start {
            index = exponent_start;
        }
    }
    // Rust's decimal conversion also maps overflow to signed infinity.
    text[..index].parse().ok()
}

fn array_row_at_max(body: &[Vec<ParseNode>], max_columns: Option<usize>) -> bool {
    max_columns.is_some_and(|maximum| body.last().is_some_and(|row| row.len() >= maximum))
}

fn unwrap_array_parse_result(
    result: Result<ParseNode, ParseError>,
    close_cell: Result<(), ParseError>,
    close_array: Result<(), ParseError>,
) -> Result<ParseNode, ParseError> {
    match (result, close_cell, close_array) {
        (Err(err), _, _) => Err(err),
        (_, Err(err), _) => Err(err),
        (_, _, Err(err)) => Err(err),
        (Ok(node), Ok(()), Ok(())) => Ok(node),
    }
}

fn is_ascii_hex_digit(c: char) -> bool {
    c.is_ascii_digit() || ('a'..='f').contains(&c) || ('A'..='F').contains(&c)
}

fn all_code_units(text: &str, predicate: impl Fn(char) -> bool) -> bool {
    if text.is_empty() {
        return false;
    }
    text.chars().all(predicate)
}

fn normalized_color(text: &str) -> Option<String> {
    if let Some(digits) = text.strip_prefix('#') {
        if (digits.len() == 3 || digits.len() == 4 || digits.len() == 6 || digits.len() == 8)
            && all_code_units(digits, is_ascii_hex_digit)
        {
            Some(text.to_string())
        } else {
            None
        }
    } else if text.chars().count() == 6 && all_code_units(text, is_ascii_hex_digit) {
        Some(format!("#{text}"))
    } else if all_code_units(text, is_ascii_alphabetic) {
        Some(text.to_string())
    } else {
        None
    }
}

fn is_url_escape_target(c: char) -> bool {
    matches!(c, '#' | '$' | '%' | '&' | '~' | '_' | '^' | '{' | '}')
}

fn unescape_url(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut builder = String::new();
    let mut segment_start = 0;
    let mut index = 0;
    while index < chars.len() {
        if chars[index] == '\\' && index + 1 < chars.len() && is_url_escape_target(chars[index + 1])
        {
            builder.extend(&chars[segment_start..index]);
            builder.push(chars[index + 1]);
            segment_start = index + 2;
            index += 2;
            continue;
        }
        index += 1;
    }
    builder.extend(&chars[segment_start..]);
    builder
}
