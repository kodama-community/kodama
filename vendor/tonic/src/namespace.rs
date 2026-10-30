use std::collections::HashMap;

use crate::error::ParseError;

/// A scoped name->value registry with undo support, mirroring MoonBit's
/// `Namespace`. Values live in `builtins` (immutable defaults) and `current`
/// (mutable); `undo_stack` records per-group changes for rollback.
pub(crate) struct Namespace<V> {
    builtins: HashMap<String, V>,
    current: HashMap<String, V>,
    undo_stack: Vec<HashMap<String, Option<V>>>,
}

impl<V> Namespace<V> {
    pub(crate) fn new(builtins: HashMap<String, V>, initial: HashMap<String, V>) -> Self {
        Namespace {
            builtins,
            current: initial,
            undo_stack: Vec::new(),
        }
    }

    pub(crate) fn begin_group(&mut self) {
        self.undo_stack.push(HashMap::new());
    }

    pub(crate) fn end_group(&mut self) -> Result<(), ParseError> {
        let Some(changes) = self.undo_stack.pop() else {
            return Err(ParseError::InternalInvariant {
                message:
                    "Unbalanced namespace destruction: attempt to pop global namespace; please report this as a bug"
                        .to_string(),
            });
        };
        self.restore_group(changes);
        Ok(())
    }

    pub(crate) fn end_groups(&mut self) {
        while let Some(changes) = self.undo_stack.pop() {
            self.restore_group(changes);
        }
    }

    pub(crate) fn has(&self, key: &str) -> bool {
        self.current.contains_key(key) || self.builtins.contains_key(key)
    }

    pub(crate) fn get(&self, key: &str) -> Option<&V> {
        self.current.get(key).or_else(|| self.builtins.get(key))
    }

    pub(crate) fn get_current(&self, key: &str) -> Option<&V> {
        self.current.get(key)
    }

    pub(crate) fn get_builtin(&self, key: &str) -> Option<&V> {
        self.builtins.get(key)
    }

    pub(crate) fn get_user_entries(&self) -> HashMap<String, &V> {
        self.current
            .iter()
            .filter(|(key, _)| !self.builtins.contains_key(*key))
            .map(|(key, value)| (key.clone(), value))
            .collect()
    }

    fn restore_group(&mut self, changes: HashMap<String, Option<V>>) {
        for (key, old_value) in changes {
            if let Some(value) = old_value {
                self.current.insert(key, value);
            } else {
                self.current.remove(&key);
            }
        }
    }

    pub(crate) fn set(&mut self, key: String, value: Option<V>, global: bool)
    where
        V: Clone,
    {
        if global {
            for changes in &mut self.undo_stack {
                changes.remove(&key);
            }
            if let Some(changes) = self.undo_stack.last_mut() {
                changes.insert(key.clone(), value.clone());
            }
        } else if let Some(changes) = self.undo_stack.last_mut() {
            changes
                .entry(key.clone())
                .or_insert_with(|| self.current.get(&key).cloned());
        }
        if let Some(v) = value {
            self.current.insert(key, v);
        } else {
            self.current.remove(&key);
        }
    }
}
