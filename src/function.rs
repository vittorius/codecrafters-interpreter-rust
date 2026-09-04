use std::fmt::Display;

use crate::{
    callable::{CallResult, Callable},
    environment::{BareEnv, Env},
    interpreter::Interpreter,
    stmt::FunctionDeclaration,
    value::Value,
};

#[derive(Debug)]
pub struct Function {
    declaration: FunctionDeclaration,
}

impl Function {
    pub fn new(declaration: FunctionDeclaration) -> Self {
        Self { declaration }
    }

    pub fn name(&self) -> &str {
        &self.declaration.name.lexeme
    }
}

impl Callable for Function {
    fn arity(&self) -> usize {
        self.declaration.params.len()
    }

    fn call(&self, interpreter: &Interpreter, arguments: &[Value], env: Env) -> CallResult {
        let env = BareEnv::with_enclosing(interpreter.globals()).wrapped();

        for (i, p) in self.declaration.params.iter().enumerate() {
            env.borrow_mut()
                .define(p.lexeme.clone(), arguments[i].clone());
        }

        interpreter.execute_block(&self.declaration.body, env)?;

        Ok(Value::Nil)
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {}>", self.declaration.name.lexeme)
    }
}
