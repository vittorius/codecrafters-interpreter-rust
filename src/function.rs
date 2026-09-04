use std::fmt::Display;

use crate::{
    callable::{CallResult, Callable},
    environment::{BareEnv, Env, clone_env},
    interpreter::Interpreter,
    stmt::FunctionDeclaration,
    value::Value,
};

#[derive(Debug)]
pub struct Function {
    declaration: FunctionDeclaration,
    closure: Env,
}

impl Function {
    pub fn new(declaration: FunctionDeclaration, closure: Env) -> Self {
        Self {
            declaration,
            closure,
        }
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
        let env = BareEnv::for_fn(clone_env(&self.closure)).wrapped();

        for (i, p) in self.declaration.params.iter().enumerate() {
            env.borrow_mut()
                .define(p.lexeme.clone(), arguments[i].clone());
        }

        interpreter.execute_block(&self.declaration.body, clone_env(&env))?;

        if let Some(return_value) = env.borrow_mut().clear_return_from_fn() {
            // The interpreter stack was naturally unwinded by the early return in the Interpreter::execute
            // AND there was an actual return value stored in the env.
            // Return the `return` value and clear the "returning" env state.
            Ok(return_value.clone())
        } else {
            Ok(Value::Nil)
        }
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {}>", self.declaration.name.lexeme)
    }
}
