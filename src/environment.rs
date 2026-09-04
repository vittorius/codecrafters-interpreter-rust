use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{error::RuntimeError, token::Token, value::Value};

pub type Env = Rc<RefCell<BareEnv>>;

// Env owns its variable names (hence String keys) to make a true REPL:
// variable definitions that survive the line of source they were derived from.
#[derive(Debug)]
pub struct BareEnv {
    values: HashMap<String, Value>,
    enclosing: Option<Env>,
}

impl BareEnv {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn with_enclosing(enclosing: Env) -> Self {
        Self {
            enclosing: Some(enclosing),
            ..Self::new()
        }
    }

    pub fn wrapped(self) -> Env {
        Rc::new(RefCell::new(self))
    }

    pub fn define(&mut self, name: &str, value: Value) {
        self.values.insert(name.to_owned(), value);
    }

    // The book throws the "undefined variable" RuntimeError right here, in the `get` method.
    // This is not very idiomatic for Rust, instead we use Option and handle this error higher up the callstack.
    // TODO: return Option<&Value> or Option<Rc<Value>> to keep Values owned by the Env only
    pub fn get(&self, name: &Token<'_>) -> Option<Value> {
        self.values.get(name.lexeme).cloned().or_else(|| {
            if let Some(enclosing) = &self.enclosing {
                enclosing.borrow().get(name)
            } else {
                None
            }
        })
    }

    pub fn assign(&mut self, name: &Token<'_>, value: Value) -> Result<Value, RuntimeError> {
        use std::collections::hash_map::Entry;

        match self.values.entry(name.lexeme.to_owned()) {
            Entry::Occupied(mut occupied_entry) => {
                occupied_entry.insert(value);
                Ok(self
                    .values
                    .get(name.lexeme)
                    .expect("A value for this key must be just inserted")
                    .clone()) // we treat Values as true "value objects" (see comment on the Value enum)
            }
            Entry::Vacant(vacant_entry) => {
                if let Some(enclosing) = &mut self.enclosing {
                    enclosing.borrow_mut().assign(name, value)
                } else {
                    Err(RuntimeError::new(
                        name,
                        &format!("Undefined variable \"{}\"", name.lexeme),
                    ))
                }
            }
        }
    }
}

// This function is added for the same explicitness as comes with calling Rc::clone
// but hiding the implementation details (`Rc`) a bit.
pub fn clone_env(env: &Env) -> Env {
    Rc::clone(env)
}
