// TODO: move inside the 'interpreter' module

use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::Display,
    rc::{Rc, Weak},
};

use crate::{class::Class, token::Token, value::Value};

pub type InstanceShared = Rc<RefCell<Instance>>;

#[derive(Debug, Clone)]
pub struct Instance {
    class: Rc<Class>,
    fields: HashMap<String, Value>,
    self_weak: Weak<RefCell<Instance>>,
}

impl Instance {
    pub fn new_shared(class: Rc<Class>) -> InstanceShared {
        Rc::new_cyclic(|weak| {
            RefCell::new(Self {
                class,
                fields: HashMap::new(),
                self_weak: weak.clone(),
            })
        })
    }

    // Returning Option<Value> as it's more Rust-idiomatic (same as in BareEnv::get()).
    // NOTE: we return Value here for the same reason as in BareEnv::get(), see notes there.
    pub fn get(&self, name: &Token) -> Option<Value> {
        if let Some(field) = self.fields.get(&name.lexeme) {
            Some(field.clone())
        } else {
            self.class
                .find_method(&name.lexeme)
                .map(|method| Value::Callable(Rc::new(method.bind(self.this()))))
        }
    }

    pub fn set(&mut self, name: Token, value: Value) {
        self.fields.insert(name.lexeme, value);
    }

    fn this(&self) -> InstanceShared {
        self.self_weak
            .upgrade()
            .expect("Getting 'this' reference for a disposed class instance")
    }
}

impl Display for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.class)
    }
}
