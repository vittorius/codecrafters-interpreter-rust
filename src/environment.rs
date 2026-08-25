use std::collections::HashMap;

use crate::{error::RuntimeError, interpreter::Value, scanner::Token};

pub struct Environment<'a> {
    values: HashMap<&'a str, Value>,
}

impl<'a> Environment<'a> {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: &Token<'a>, value: Value) {
        self.values.insert(name.lexeme, value);
    }

    // The book throws the "undefined variable" RuntimeError right here, in the `get` method.
    // This is not very idiomatic for Rust, instead we use Option and handle this error higher up the callstack.
    pub fn get(&self, name: &Token<'a>) -> Option<&Value> {
        self.values.get(name.lexeme)
    }

    pub fn assign(&mut self, name: &Token<'a>, value: Value) -> Result<&Value, RuntimeError> {
        use std::collections::hash_map::Entry;

        match self.values.entry(name.lexeme) {
            Entry::Occupied(mut occupied_entry) => {
                occupied_entry.insert(value);
                Ok(self
                    .values
                    .get(name.lexeme)
                    .expect("A value for this key must be just inserted"))
            }
            Entry::Vacant(vacant_entry) => Err(RuntimeError::new(
                name,
                &format!("Undefined variable \"{}\"", name.lexeme),
            )),
        }
    }
}
