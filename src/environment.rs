use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{error::RuntimeError, interpreter::Value, scanner::Token};

// TODO: replace Value with Rc<Value> to avoid cloning values when visiting variables and assignments
pub struct Environment<'a> {
    values: HashMap<&'a str, Value>,
    // values: HashMap<&'a str, Rc<Value>>,
    // enclosing: Option<&'a mut Environment<'a>>,
    // enclosing: Option<&'a mut Environment<'a>>,
    enclosing: Option<Rc<RefCell<Environment<'a>>>>,
    // enclosing: Option<Rc<Environment<'a>>>,
}

impl<'a> Environment<'a> {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    // pub fn with_enclosing(enclosing: &'a mut Environment<'a>) -> Self {
    pub fn with_enclosing(enclosing: Rc<RefCell<Environment<'a>>>) -> Self {
        // pub fn with_enclosing(enclosing: Rc<Environment<'a>>) -> Self {
        Self {
            enclosing: Some(enclosing),
            ..Self::new()
        }
    }

    pub fn define(&mut self, name: &'a Token<'a>, value: Value) {
        self.values.insert(name.lexeme, value);
    }

    // The book throws the "undefined variable" RuntimeError right here, in the `get` method.
    // This is not very idiomatic for Rust, instead we use Option and handle this error higher up the callstack.
    pub fn get(&self, name: &'a Token<'a>) -> Option<Value> {
        // pub fn get(&self, name: &Token<'a>) -> Option<Rc<Value>> {
        if let Some(enclosing) = &self.enclosing {
            // enclosing.borrow().get(name)
            enclosing.get(name)
        } else {
            self.values.get(name.lexeme).cloned() // we treat Values as true "value objects" (see comment on the Value enum)
        }
    }

    pub fn assign(&mut self, name: &'a Token<'a>, value: Value) -> Result<Value, RuntimeError> {
        use std::collections::hash_map::Entry;

        if let Some(enclosing) = &mut self.enclosing {
            // return enclosing.borrow_mut().assign(name, value);
            return enclosing.assign(name, value);
        }

        match self.values.entry(name.lexeme) {
            Entry::Occupied(mut occupied_entry) => {
                occupied_entry.insert(value);
                Ok(self
                    .values
                    .get(name.lexeme)
                    .expect("A value for this key must be just inserted")
                    .clone()) // we treat Values as true "value objects" (see comment on the Value enum)
            }
            Entry::Vacant(vacant_entry) => Err(RuntimeError::new(
                name,
                &format!("Undefined variable \"{}\"", name.lexeme),
            )),
        }
    }
}
