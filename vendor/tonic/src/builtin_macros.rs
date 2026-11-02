use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::Mode;
use crate::macro_expander::MacroHandler;
use crate::symbol_registry::lookup_symbol;

use crate::builtin_macros_commands::*;
use crate::builtin_macros_control::*;
use crate::builtin_macros_dots::*;
use crate::builtin_macros_special::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MathSymbolGroup {
    BinarySymbol,
    RelationSymbol,
}

pub(crate) type MathSymbolGroupResolver = Rc<dyn Fn(&str) -> Option<MathSymbolGroup>>;

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum MacroLogEvent {
    MacroMessage(String),
    MacroErrorMessage(String),
    MacroShow(String),
}

pub(crate) type MacroReporter = Rc<dyn Fn(MacroLogEvent)>;

pub(crate) fn default_math_symbol_group(name: &str) -> Option<MathSymbolGroup> {
    match lookup_symbol(Mode::Math, name) {
        Some(symbol) if symbol.group == crate::symbol_registry::SymbolGroup::BinaryGroup => {
            Some(MathSymbolGroup::BinarySymbol)
        }
        Some(symbol) if symbol.group == crate::symbol_registry::SymbolGroup::RelationGroup => {
            Some(MathSymbolGroup::RelationSymbol)
        }
        _ => None,
    }
}

pub(crate) fn default_macro_reporter(event: MacroLogEvent) {
    match event {
        MacroLogEvent::MacroMessage(message)
        | MacroLogEvent::MacroErrorMessage(message)
        | MacroLogEvent::MacroShow(message) => println!("{message}"),
    }
}

pub(crate) fn builtin_dynamic_macros() -> HashMap<String, MacroHandler> {
    let mut map: HashMap<String, MacroHandler> = HashMap::new();
    map.insert("\\noexpand".to_string(), Rc::new(noexpand_macro));
    map.insert("\\expandafter".to_string(), Rc::new(expandafter_macro));
    map.insert("\\@firstoftwo".to_string(), Rc::new(first_of_two_macro));
    map.insert("\\@secondoftwo".to_string(), Rc::new(second_of_two_macro));
    map.insert("\\@ifnextchar".to_string(), Rc::new(if_next_char_macro));
    map.insert("\\TextOrMath".to_string(), Rc::new(text_or_math_macro));
    map.insert("\\char".to_string(), Rc::new(char_macro));
    map.insert("\\newcommand".to_string(), Rc::new(new_command_macro));
    map.insert("\\renewcommand".to_string(), Rc::new(renew_command_macro));
    map.insert(
        "\\providecommand".to_string(),
        Rc::new(provide_command_macro),
    );
    map.insert("\\message".to_string(), Rc::new(message_macro));
    map.insert("\\errmessage".to_string(), Rc::new(error_message_macro));
    map.insert("\\show".to_string(), Rc::new(show_macro));
    map.insert("\\dots".to_string(), Rc::new(dots_macro));
    map.insert("\\dotso".to_string(), Rc::new(dots_other_macro));
    map.insert("\\dotsc".to_string(), Rc::new(dots_comma_macro));
    map.insert("\\cdots".to_string(), Rc::new(centered_dots_macro));
    map.insert("\\tag@literal".to_string(), Rc::new(tag_literal_macro));
    map.insert("\\bra@ket".to_string(), Rc::new(braket_macro));
    map.insert("\\bra@set".to_string(), Rc::new(set_macro));
    map
}
