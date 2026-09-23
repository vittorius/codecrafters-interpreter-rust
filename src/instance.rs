// TODO: move inside the 'interpreter' module

use std::{collections::HashMap, fmt::Display, rc::Rc, time::{SystemTime, UNIX_EPOCH}};

use crate::{class::Class, error::RuntimeError, token::Token, value::Value};

// TODO: use Result<&Value, RuntimeError> to delegate cloning to the caller code and replicate HashMap API
type PropertyAccessResult = std::result::Result<Value, RuntimeError>;

#[derive(Debug, Clone)]
pub struct Instance {
    class: Rc<Class>,
    fields: HashMap<String, Value>,
    id: f64,
}

impl Instance {
    pub fn new(class: Rc<Class>) -> Self {
        eprintln!("Creating a new instance of {}", class);
        Self {
            class,
            fields: HashMap::new(),
            id: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .expect("system time is before the Unix epoch")
                            .as_secs_f64(),
        }
    }

    pub fn get(&self, name: &Token) -> PropertyAccessResult {
        eprintln!("self {:?}, self.fields: {:#?}", self.id, self.fields);
        self.fields
            .get(&name.lexeme)
            .cloned()
            .ok_or(RuntimeError::new(
                name,
                &format!("Undefined property '{}'.", name.lexeme),
            ))
    }

    pub fn set(&mut self, name: Token, value: Value) {
        eprintln!("self: {:?}, name: {}, value: {}", self.id, &name.lexeme, &value);
        self.fields.insert(name.lexeme, value);
        eprintln!("self.fields: {:#?}", self.fields);
    }
}

impl Display for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.class)
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        eprintln!("Instance {} being dropped", self.id);
    }
}
