// TODO: move inside the 'interpreter' module

use std::{collections::HashMap, fmt::Display, rc::Rc};

use crate::{
    callable::{CallResult, Callable},
    function::Function,
    instance::Instance,
    interpreter::Interpreter,
    token::Token,
    value::Value,
};

// TODO: type ClassShared

#[derive(Debug)]
pub struct Class {
    name: Token,
    methods: HashMap<String, Rc<Function>>,
}

impl Class {
    pub fn new(name: Token, methods: HashMap<String, Rc<Function>>) -> Self {
        Self { name, methods }
    }

    pub fn find_method(&self, name: &str) -> Option<Rc<Function>> {
        self.methods.get(name).map(Rc::clone)
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
        Ok(Value::Object(Instance::new_shared(Rc::clone(&self))))
    }
}
