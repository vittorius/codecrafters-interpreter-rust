// TODO: move inside the 'interpreter' module

use std::{cell::RefCell, collections::HashMap, num::NonZeroUsize, rc::Rc};

use crate::{error::RuntimeError, token::Token, value::Value};

// Rc<RefCell<...>> usage is inevitable because a single environment can become primary or enclosing
// for multiple child environments where it can be potentially mutated (e.g. Binary expression)
pub type Env = Rc<RefCell<BareEnv>>;

// Env owns its variable names (hence String keys) to make a true REPL:
// variable definitions that survive the line of source they were derived from.
#[derive(Debug)]
pub struct BareEnv {
    values: HashMap<String, Value>,
    enclosing: Option<Env>,
    return_value: Option<Value>,
}

impl BareEnv {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            enclosing: None,
            return_value: None,
        }
    }

    pub fn with_enclosing(enclosing: Env) -> Self {
        Self {
            enclosing: Some(enclosing),
            values: HashMap::new(),
            return_value: None,
        }
    }

    pub fn wrapped(self) -> Env {
        Rc::new(RefCell::new(self))
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    // The book throws the "undefined variable" RuntimeError right here, in the `get` method.
    // This is not very idiomatic for Rust, instead we use Option and handle this error higher up the callstack.
    // TODO: return Option<&Value> or Option<Rc<Value>> to keep Values owned by the Env only
    // TODO: revisit having &str as the key here instead of a &Token
    pub fn get(&self, name: &Token) -> Option<Value> {
        self.values.get(&name.lexeme).cloned().or_else(|| {
            if let Some(enclosing) = &self.enclosing {
                enclosing.borrow().get(name)
            } else {
                None
            }
        })
    }

    // TODO: return Option<&Value> or Option<Rc<Value>> to keep Values owned by the Env only
    // TODO: revisit having &str as the key here instead of a &Token
    pub fn get_at(&self, distance: usize, name: &Token) -> Option<Value> {
        if distance == 0 {
            self.get(name)
        } else {
            self.ancestor(NonZeroUsize::new(distance).expect("The distance must be non-zero"))
                .borrow()
                .values
                .get(&name.lexeme)
                .cloned()
        }
    }

    // TODO: consume `name` and make the caller .clone()
    pub fn assign(&mut self, name: &Token, value: Value) -> Result<Value, RuntimeError> {
        use std::collections::hash_map::Entry;

        // TODO: try using .contains_key() to avoid premature cloning of `name.lexeme`
        match self.values.entry(name.lexeme.clone()) {
            Entry::Occupied(mut occupied_entry) => {
                occupied_entry.insert(value.clone()); // TODO: avoid this clone making the caller do this
                Ok(value)
            }
            Entry::Vacant(_) => {
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

    // TODO: consume `name` and make the caller .clone()
    pub fn assign_at(
        &mut self,
        distance: usize,
        name: &Token,
        value: Value,
    ) -> Result<Value, RuntimeError> {
        if distance == 0 {
            self.assign(name, value)
        } else {
            self.ancestor(NonZeroUsize::new(distance).expect("The distance must be non-zero"))
                .borrow_mut()
                .values
                .insert(name.lexeme.clone(), value.clone());
            Ok(value)
        }
    }

    pub fn return_from_fn(&mut self, value: Value) {
        self.return_value = Some(value);
    }

    pub fn is_returning_from_fn(&self) -> bool {
        self.return_value.is_some()
    }

    pub fn clear_return_from_fn(&mut self) -> Option<Value> {
        self.return_value.take()
    }

    fn ancestor(&self, distance: NonZeroUsize) -> Env {
        let mut env = clone_env(
            self.enclosing
                .as_ref()
                .expect("Enclosing env must be present (trusting the resolver)"),
        );

        for _ in NonZeroUsize::MIN..distance {
            let env_clone = clone_env(
                env.borrow()
                    .enclosing
                    .as_ref()
                    .expect("Enclosing env must be present (trusting the resolver)"),
            );
            env = env_clone;
        }

        env
    }
}

// This function is added for the same explicitness as comes with calling Rc::clone
// but hiding the implementation details (`Rc`) a bit.
pub fn clone_env(env: &Env) -> Env {
    Rc::clone(env)
}

impl Drop for BareEnv {
    fn drop(&mut self) {
        if let Some(enclosing) = &self.enclosing
            && let Some(return_value) = self.return_value.take()
        {
            enclosing.borrow_mut().return_from_fn(return_value);
        }
    }
}
