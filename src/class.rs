// TODO: move inside the 'interpreter' module

use std::{fmt::Display, rc::Rc};

use crate::{
    callable::{CallResult, Callable},
    instance::Instance,
    interpreter::Interpreter,
    token::Token,
    value::Value,
};

#[derive(Debug)]
pub struct Class {
    name: Token,
}

impl Class {
    pub fn new(name: Token) -> Self {
        Self { name }
    }
}

impl Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name.lexeme)
    }
}

impl Callable for Class {
    fn arity(&self) -> usize {
        0
    }

    fn call(self: Rc<Self>, _interpreter: &Interpreter, _arguments: &[Value]) -> CallResult {
        Ok(Value::Object(Instance::new(Rc::clone(&self))))
    }
}
