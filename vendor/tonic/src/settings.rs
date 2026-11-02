use std::collections::HashMap;
use std::rc::Rc;

use crate::error::ParseError;
use crate::macro_definition::MacroDefinition;
use crate::source_location::SourceLocation;
use crate::token::{Token, token_location};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The result of a strict-mode check: ignore, warn, or error.
pub enum StrictResponse {
    Ignore,
    Warn,
    Error,
}

pub type StrictHandler =
    Rc<dyn Fn(&str, &str, Option<&SourceLocation>) -> Result<StrictResponse, ParseError>>;

/// The strictness policy for LaTeX-incompatible input.
pub enum Strictness {
    Ignore,
    Warn,
    Error,
    Callback(StrictHandler),
}

impl Clone for Strictness {
    fn clone(&self) -> Self {
        match self {
            Strictness::Ignore => Strictness::Ignore,
            Strictness::Warn => Strictness::Warn,
            Strictness::Error => Strictness::Error,
            Strictness::Callback(handler) => Strictness::Callback(handler.clone()),
        }
    }
}

/// A callback invoked when strict mode produces a warning.
pub type StrictWarningHandler = Rc<dyn Fn(&str)>;

/// The context of a trust check for potentially unsafe commands.
pub enum TrustContext {
    UrlTrust {
        command: String,
        url: String,
        protocol: Option<String>,
    },
    HtmlClass {
        class: String,
    },
    HtmlId {
        id: String,
    },
    HtmlStyle {
        style: String,
    },
    HtmlData {
        attributes: HashMap<String, String>,
    },
}

/// A callback deciding whether a trust context is accepted.
pub type TrustHandler = Rc<dyn Fn(&TrustContext) -> bool>;

/// The trust policy for commands that could be unsafe (\href, \url, HTML).
pub enum TrustPolicy {
    Untrusted,
    Trusted,
    Callback(TrustHandler),
}

impl Clone for TrustPolicy {
    fn clone(&self) -> Self {
        match self {
            TrustPolicy::Untrusted => TrustPolicy::Untrusted,
            TrustPolicy::Trusted => TrustPolicy::Trusted,
            TrustPolicy::Callback(handler) => TrustPolicy::Callback(handler.clone()),
        }
    }
}

#[derive(Debug, Clone)]
/// A persistent macro store shared across parses.
pub struct Macros(pub HashMap<String, MacroDefinition>);

impl Macros {
    pub fn new(macros: HashMap<String, String>) -> Self {
        let definitions = macros
            .into_iter()
            .map(|(name, expansion)| (name, MacroDefinition::text(expansion)))
            .collect();
        Macros(definitions)
    }
}

#[derive(Clone)]
/// Parser configuration: display mode, macros, strictness, trust, and limits.
pub struct Settings {
    pub throw_on_error: bool,
    pub display_mode: bool,
    pub leqno: bool,
    pub error_color: String,
    pub color_is_text_color: bool,
    pub max_expand: usize,
    pub global_group: bool,
    pub macros: HashMap<String, String>,
    pub macro_store: Option<Macros>,
    pub strict: Strictness,
    pub strict_warning_handler: Option<StrictWarningHandler>,
    pub trust: TrustPolicy,
}

impl Settings {
    pub fn new() -> Self {
        Settings {
            throw_on_error: true,
            display_mode: false,
            leqno: false,
            error_color: "#cc0000".to_string(),
            color_is_text_color: false,
            max_expand: 1000,
            global_group: false,
            macros: HashMap::new(),
            macro_store: None,
            strict: Strictness::Ignore,
            strict_warning_handler: None,
            trust: TrustPolicy::Untrusted,
        }
    }

    pub fn macro_definitions(&self) -> HashMap<String, MacroDefinition> {
        self.macros
            .iter()
            .map(|(name, expansion)| (name.clone(), MacroDefinition::text(expansion.clone())))
            .collect()
    }

    pub fn use_strict_behavior(
        &self,
        error_code: &str,
        error_message: &str,
        token: Option<&Token>,
    ) -> bool {
        match strict_response(
            &self.strict,
            error_code,
            error_message,
            token_location(token),
        ) {
            StrictResponse::Error => true,
            StrictResponse::Warn => {
                let warning = format!(
                    "LaTeX-incompatible input and strict mode is set to 'warn': {error_message} [{error_code}]"
                );
                if let Some(handler) = &self.strict_warning_handler {
                    handler(&warning);
                }
                false
            }
            StrictResponse::Ignore => false,
        }
    }

    pub fn report_nonstrict(
        &self,
        error_code: &str,
        error_message: &str,
        token: Option<&Token>,
    ) -> Result<(), ParseError> {
        match strict_response(
            &self.strict,
            error_code,
            error_message,
            token_location(token),
        ) {
            StrictResponse::Ignore => Ok(()),
            StrictResponse::Warn => {
                let warning = format!(
                    "LaTeX-incompatible input and strict mode is set to 'warn': {error_message} [{error_code}]"
                );
                if let Some(handler) = &self.strict_warning_handler {
                    handler(&warning);
                }
                Ok(())
            }
            StrictResponse::Error => Err(ParseError::InvalidArgument {
                message: format!(
                    "LaTeX-incompatible input and strict mode is set to 'error': {error_message} [{error_code}]"
                ),
                loc: token_location(token),
            }),
        }
    }

    pub fn is_trusted(&self, context: TrustContext) -> bool {
        let context = if let TrustContext::UrlTrust { command, url, .. } = context {
            let Some(protocol) = url_protocol(&url) else {
                return false;
            };
            TrustContext::UrlTrust {
                command,
                url,
                protocol: Some(protocol),
            }
        } else {
            context
        };
        match &self.trust {
            TrustPolicy::Untrusted => false,
            TrustPolicy::Trusted => true,
            TrustPolicy::Callback(handler) => handler(&context),
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Settings::new()
    }
}

fn strict_response(
    strictness: &Strictness,
    error_code: &str,
    error_message: &str,
    loc: Option<SourceLocation>,
) -> StrictResponse {
    match strictness {
        Strictness::Ignore => StrictResponse::Ignore,
        Strictness::Warn => StrictResponse::Warn,
        Strictness::Error => StrictResponse::Error,
        Strictness::Callback(handler) => {
            handler(error_code, error_message, loc.as_ref()).unwrap_or(StrictResponse::Error)
        }
    }
}

fn ascii_lower(text: &str) -> String {
    text.chars().map(|c| c.to_ascii_lowercase()).collect()
}

fn url_starts_with(url: &[char], offset: usize, prefix: &str) -> bool {
    let prefix: Vec<char> = prefix.chars().collect();
    let prefix_len = prefix.len();
    offset + prefix_len <= url.len() && url[offset..offset + prefix_len] == prefix[..]
}

fn encoded_colon_at(url: &[char], offset: usize) -> bool {
    let lower: Vec<char> = url.iter().map(char::to_ascii_lowercase).collect();
    if url_starts_with(&lower, offset, "&colon") {
        return true;
    }
    if !url_starts_with(url, offset, "&#") {
        return false;
    }
    let mut index = offset + 2;
    if index < url.len() && (url[index] == 'x' || url[index] == 'X') {
        index += 1;
        while index < url.len() && url[index] == '0' {
            index += 1;
        }
        url_starts_with(&lower, index, "3a")
    } else {
        while index < url.len() && url[index] == '0' {
            index += 1;
        }
        url_starts_with(url, index, "58")
    }
}

fn url_protocol(url: &str) -> Option<String> {
    let chars: Vec<char> = url.chars().collect();
    let len = chars.len();
    let mut index = 0;
    while index < len && (chars[index] as u32) <= 0x20 {
        index += 1;
    }
    let start = index;
    while index < len {
        let code = chars[index];
        if code == ':' {
            if index <= start {
                return None;
            }
            let scheme: String = chars[start..index].iter().collect();
            if !scheme
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_alphabetic())
            {
                return None;
            }
            for c in scheme.chars() {
                if !(c.is_ascii_alphabetic()
                    || c.is_ascii_digit()
                    || c == '+'
                    || c == '-'
                    || c == '.')
                {
                    return None;
                }
            }
            return Some(ascii_lower(&scheme));
        }
        if code == '/' || code == '#' || code == '?' {
            return Some("_relative".to_string());
        }
        if code == '&' && encoded_colon_at(&chars, index) {
            return None;
        }
        index += 1;
    }
    Some("_relative".to_string())
}
