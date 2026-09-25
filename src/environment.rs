// TODO: move inside the 'interpreter' module

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{error::RuntimeError, token::Token, value::Value};

// The approach with Env/EnvData is borrowed from here https://github.com/cc-code-examples/kind-leopard-632316/blob/main/src/environment.rs#L8
// Rc<RefCell<...>> usage is inevitable because a single environment can become primary or enclosing
// for multiple child environments where it can be potentially mutated (e.g. when evaluating a Binary expression)

#[derive(Debug)]
pub struct Env {
    data: Rc<RefCell<EnvData>>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            data: Rc::new(RefCell::new(EnvData::default())),
        }
    }

    pub fn new_enclosed_with(enclosing: &Env) -> Self {
        Self {
            data: Rc::new(RefCell::new(EnvData::new_enclosed_with(Rc::clone(
                &enclosing.data,
            )))),
        }
    }

    pub fn define(&self, name: String, value: Value) {
        self.data.borrow_mut().values.insert(name, value);
    }

    // The book throws the "undefined variable" RuntimeError right here, in the `get` method.
    // This is not very idiomatic for Rust, instead we use Option and handle this error higher up the callstack.
    //
    // It's not possible to return Option<&Value> because &Value cannot outlive the output of enclosing.borrow().
    // The environment could be HashMap<String, Rc<RefCell<Value>>> but it seems more natural to move the
    // value/reference duality to the Value itself (see Value definition.)
    pub fn get(&self, name: &str) -> Option<Value> {
        self.data.borrow().get(name)
    }

    // See the `get` method note about not returning Option<&Value> here.
    pub fn get_at(&self, distance: usize, name: &str) -> Option<Value> {
        self.data.borrow().get_at(distance, name)
    }

    // See the `get` method note about not returning Result<&Value, _> here.
    pub fn assign(&self, name: &Token, value: Value) -> Result<Value, RuntimeError> {
        self.data.borrow_mut().assign(name, value)
    }

    // See the `get` method note about not returning Result<&Value, _> here.
    pub fn assign_at(
        &self,
        distance: usize,
        name: &Token,
        value: Value,
    ) -> Result<Value, RuntimeError> {
        self.data.borrow_mut().assign_at(distance, name, value)
    }

    pub fn set_return_from_fn(&self, value: Value) {
        self.data.borrow_mut().return_value = Some(value);
    }

    pub fn is_returning_from_fn(&self) -> bool {
        self.data.borrow().return_value.is_some()
    }

    pub fn take_return_from_fn(&self) -> Option<Value> {
        self.data.borrow_mut().return_value.take()
    }
}

impl Clone for Env {
    // Use Env::clone() syntax (like Rc::clone()) to emphasize the ref-cloning nature of this operation.
    fn clone(&self) -> Self {
        Self {
            data: Rc::clone(&self.data),
        }
    }
}

impl Drop for Env {
    fn drop(&mut self) {
        if let Some(enclosing) = &self.data.borrow().enclosing
            && let Some(return_value) = self.data.borrow().return_value.as_ref()
        {
            enclosing.borrow_mut().return_value = Some(return_value.clone()); // the original value will be dropped
        }
    }
}

#[derive(Debug, Default)]
struct EnvData {
    // Environment owns its variable names (hence String keys) to make a true REPL:
    // variable definitions that survive the line of source they were derived from.
    values: HashMap<String, Value>,
    enclosing: Option<Rc<RefCell<EnvData>>>,
    return_value: Option<Value>,
}

impl EnvData {
    fn new_enclosed_with(env: Rc<RefCell<EnvData>>) -> Self {
        Self {
            enclosing: Some(env),
            ..EnvData::default()
        }
    }

    fn get(&self, name: &str) -> Option<Value> {
        if let Some(value) = self.values.get(name) {
            Some(value.clone())
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.borrow().get(name)
        } else {
            None
        }
    }

    fn get_at(&self, distance: usize, name: &str) -> Option<Value> {
        if distance == 0 {
            self.get(name)
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.borrow().get_at(distance - 1, name)
        } else {
            unreachable!(
                "Wrong var resolution distance {} when enclosing env is missing",
                distance
            )
        }
    }

    fn assign(&mut self, name: &Token, value: Value) -> Result<Value, RuntimeError> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.clone(), value.clone());
            Ok(value)
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.borrow_mut().assign(name, value)
        } else {
            Err(RuntimeError::new(
                name,
                &format!("Undefined variable \"{}\"", name.lexeme),
            ))
        }
    }

    fn assign_at(
        &mut self,
        distance: usize,
        name: &Token,
        value: Value,
    ) -> Result<Value, RuntimeError> {
        if distance == 0 {
            self.assign(name, value)
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.borrow_mut().assign_at(distance - 1, name, value)
        } else {
            unreachable!(
                "Wrong var resolution distance {} when enclosing env is missing",
                distance
            )
        }
    }
}
