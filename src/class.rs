// TODO: move inside the 'interpreter' module

use std::fmt::Display;

use crate::{callable::Callable, token::Token};

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
        todo!()
    }

    fn call(
        &self,
        interpreter: &crate::interpreter::Interpreter,
        arguments: &[crate::value::Value],
        env: crate::environment::Env,
    ) -> crate::callable::CallResult {
        todo!()
    }
}
