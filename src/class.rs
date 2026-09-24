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
        if let Some(initializer) = self.find_method("init") {
            initializer.arity()
        } else {
            0
        }
    }

    // using Rc<Self> here as a receiver type to make the 'self' reference escape into newly created Instance
    fn call(self: Rc<Self>, interpreter: &Interpreter, arguments: &[Value]) -> CallResult {
        let instance = Instance::new_shared(Rc::clone(&self));

        if let Some(initializer) = self.find_method("init") {
            let initializer = Rc::new(initializer.bind(Rc::clone(&instance)));
            initializer.call(interpreter, arguments)?;
        }

        Ok(Value::Object(instance))
    }
}
