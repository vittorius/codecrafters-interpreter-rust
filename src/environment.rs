use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{error::RuntimeError, interpreter::Value, scanner::Token};

pub type Env<'a> = Rc<RefCell<BareEnv<'a>>>;

pub fn clone_env<'a>(env: &Env<'a>) -> Env<'a> {
    Rc::clone(env)
}

// TODO: replace Value with Rc<Value> to avoid cloning values when visiting variables and assignments
#[derive(Debug)]
pub struct BareEnv<'a> {
    values: HashMap<&'a str, Value>,
    // values: HashMap<&'a str, Rc<Value>>,
    // enclosing: Option<&'a mut Environment<'a>>,
    // enclosing: Option<&'a mut Environment<'a>>,
    enclosing: Option<Rc<RefCell<BareEnv<'a>>>>,
    // enclosing: Option<Rc<Environment<'a>>>,
}

impl<'a> BareEnv<'a> {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    // pub fn with_enclosing(enclosing: &'a mut Environment<'a>) -> Self {
    pub fn with_enclosing(enclosing: Rc<RefCell<BareEnv<'a>>>) -> Self {
        // pub fn with_enclosing(enclosing: Rc<Environment<'a>>) -> Self {
        Self {
            enclosing: Some(enclosing),
            ..Self::new()
        }
    }

    pub fn wrapped(self) -> Rc<RefCell<BareEnv<'a>>> {
        Rc::new(RefCell::new(self))
    }

    pub fn define(&mut self, name: &'a Token<'a>, value: Value) {
        self.values.insert(name.lexeme, value);
    }

    // The book throws the "undefined variable" RuntimeError right here, in the `get` method.
    // This is not very idiomatic for Rust, instead we use Option and handle this error higher up the callstack.
    pub fn get(&self, name: &'a Token<'a>) -> Option<Value> {
        self.values.get(name.lexeme).cloned().or_else(|| {
            if let Some(enclosing) = &self.enclosing {
                enclosing.borrow().get(name)
            } else {
                None
            }
        })
    }

    pub fn assign(&mut self, name: &'a Token<'a>, value: Value) -> Result<Value, RuntimeError> {
        use std::collections::hash_map::Entry;

        if let Some(enclosing) = &mut self.enclosing {
            return enclosing.borrow_mut().assign(name, value);
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
