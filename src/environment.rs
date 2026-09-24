// TODO: move inside the 'interpreter' module

use std::{cell::RefCell, collections::HashMap, num::NonZeroUsize, rc::Rc};

use crate::{error::RuntimeError, token::Token, value::Value};

// TODO: implement the approach with Environment/EnvironmentData from here https://github.com/cc-code-examples/kind-leopard-632316/blob/main/src/environment.rs#L8
// to encapsulate borrow/borrow_mut calls inside the environment
// Rc<RefCell<...>> usage is inevitable because a single environment can become primary or enclosing
// for multiple child environments where it can be potentially mutated (e.g. Binary expression)
// TODO: rename into EnvShared
pub type EnvShared = Rc<RefCell<Env>>;

// Env owns its variable names (hence String keys) to make a true REPL:
// variable definitions that survive the line of source they were derived from.
#[derive(Debug)]
// TODO: rename into Env
pub struct Env {
    // values: HashMap<String, Value>,
    // enclosing: Option<EnvShared>,
    // return_value: Option<Value>,
    data: Rc<RefCell<EnvData>>,
}

#[derive(Debug, Default)]
struct EnvData {
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
}

impl Env {
    // TODO: delete this method and move its logic to `wrapped`
    pub fn new() -> Self {
        Self {
            data: Rc::new(RefCell::new(EnvData::default())),
        }
    }

    // TODO: rename to enclosed_with
    pub fn with_enclosing(enclosing: &Env) -> Self {
        Self {
            data: Rc::new(RefCell::new(EnvData::new_enclosed_with(Rc::clone(
                &enclosing.data,
            )))),
        }
    }

    // TODO: rename to `new_shared`
    // pub fn wrapped(self) -> EnvShared {
    //     Rc::new(RefCell::new(self))
    // }

    pub fn define(&mut self, name: String, value: Value) {
        self.data.borrow_mut().values.insert(name, value);
    }

    // TODO: use when BareEnv is renamed to Env
    // pub fn clone_shared(env: &Env) -> Env {
    //     Rc::clone(&env)
    // }

    // The book throws the "undefined variable" RuntimeError right here, in the `get` method.
    // This is not very idiomatic for Rust, instead we use Option and handle this error higher up the callstack.
    //
    // It's not possible to return Option<&Value> because &Value cannot outlive the output of enclosing.borrow().
    // The environment could be HashMap<String, Rc<RefCell<Value>>> but it seems more natural to move the
    // value/reference duality to the Value itself (see Value definition.)
    pub fn get(&self, name: &str) -> Option<Value> {
        // self.values.get(name).cloned().or_else(|| {
        //     if let Some(enclosing) = &self.enclosing {
        //         enclosing.borrow().get(name)
        //     } else {
        //         None
        //     }
        // })
        // self.data.borrow().values.get(name).or_else(|| {
        //     if let Some(enclosing) = s
        // })
        self.data.borrow().get(name)
    }

    // See the `get` method note about not returning Option<&Value> here.
    pub fn get_at(&self, distance: usize, name: &str) -> Option<Value> {
        // if distance == 0 {
        //     self.get(name)
        // } else {
        //     self.ancestor(NonZeroUsize::new(distance).expect("The distance must be non-zero"))
        //         .borrow()
        //         .values
        //         .get(name)
        //         .cloned()
        // }
        self.data.borrow().get_at(distance, name)
    }

    // TODO: return Result<(), RuntimeError> here and make the caller mess with Value cloning
    // Using &Token here is correct because this method semantics is similar to .get() or .contains_key():
    // you cannot assign what hasn't been defined, so .assign() must not consume the key but rather borrow it.
    //     pub fn assign(&mut self, name: &Token, value: Value) -> Result<Value, RuntimeError> {
    //         use std::collections::hash_map::Entry;
    //
    //         // TODO: try using .contains_key() to avoid premature cloning of `name.lexeme`
    //         match self.values.entry(name.lexeme.clone()) {
    //             Entry::Occupied(mut occupied_entry) => {
    //                 occupied_entry.insert(value.clone()); // TODO: avoid this clone making the caller do this
    //                 Ok(value)
    //             }
    //             Entry::Vacant(_) => {
    //                 if let Some(enclosing) = &mut self.enclosing {
    //                     enclosing.borrow_mut().assign(name, value)
    //                 } else {
    //                     Err(RuntimeError::new(
    //                         name,
    //                         &format!("Undefined variable \"{}\"", name.lexeme),
    //                     ))
    //                 }
    //             }
    //         }
    //     }

    pub fn assign(&self, name: &Token, value: Value) -> Result<Value, RuntimeError> {
        self.data.borrow_mut().assign(name, value)
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

    fn ancestor(&self, distance: NonZeroUsize) -> EnvShared {
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
pub fn clone_env(env: &EnvShared) -> EnvShared {
    Rc::clone(env)
}

impl Drop for Env {
    fn drop(&mut self) {
        if let Some(enclosing) = &self.enclosing
            && let Some(return_value) = self.return_value.take()
        {
            enclosing.borrow_mut().return_from_fn(return_value);
        }
    }
}
