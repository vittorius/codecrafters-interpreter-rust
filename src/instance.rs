// TODO: move inside the 'interpreter' module

use std::{collections::HashMap, fmt::Display, rc::Rc};

use crate::{class::Class, error::RuntimeError, token::Token, value::Value};

// TODO: use Result<&Value, RuntimeError> to delegate cloning to the caller code and replicate HashMap API
type PropertyAccessResult = std::result::Result<Value, RuntimeError>;

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

    pub fn get(&self, name: &Token) -> PropertyAccessResult {
        self.fields
            .get(&name.lexeme)
            .cloned()
            .ok_or(RuntimeError::new(name, "Only instances have properties."))
    }
}

impl Display for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.class)
    }
}
