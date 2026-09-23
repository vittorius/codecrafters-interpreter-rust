// TODO: move inside the 'interpreter' module

use std::{collections::HashMap, fmt::Display, rc::Rc};

use crate::{class::Class, token::Token, value::Value};

#[derive(Debug, Clone)]
pub struct Instance {
    class: Rc<Class>,
    fields: HashMap<String, Value>,
}

impl Instance {
    pub fn new(class: Rc<Class>) -> Self {
        Self {
            class,
            fields: HashMap::new(),
        }
    }

    // Returning Option<Value> as it's more Rust-idiomatic (same as in BareEnv::get()).
    // NOTE: we return Value here for the same reason as in BareEnv::get(), see notes there.
    pub fn get(&self, name: &Token) -> Option<Value> {
        if let Some(field) = self.fields.get(&name.lexeme) {
            Some(field.clone())
        } else {
            self.class
                .find_method(&name.lexeme)
                .map(|method| Value::Callable(method))
        }
    }

    pub fn set(&mut self, name: Token, value: Value) {
        self.fields.insert(name.lexeme, value);
    }
}

impl Display for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.class)
    }
}
