use std::fmt::Display;

use crate::{
    callable::{CallResult, Callable},
    environment::{BareEnv, Env, clone_env},
    expr::fun_expr::FunExpr,
    interpreter::Interpreter,
    stmt::fun_decl::FunDecl,
    token::Token,
    value::Value,
};

// TODO: use const generics to separate function and lambda implementation details
// Or, have 2 different types for function and lambda and move call default impl
// to the Callable trait body
#[derive(Debug)]
pub struct Function {
    name: Option<Token>,
    fun_expr: FunExpr,
    closure: Env,
}

impl Function {
    pub fn new(decl: FunDecl, closure: Env) -> Self {
        Self {
            name: Some(decl.name),
            fun_expr: decl.expr,
            closure,
        }
    }

    #[cfg(feature = "lambda")]
    pub fn new_lambda(fun_expr: FunExpr, closure: Env) -> Self {
        Self {
            name: None,
            fun_expr,
            closure,
        }
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(|t| t.lexeme.as_str())
    }
}

impl Callable for Function {
    fn arity(&self) -> usize {
        self.fun_expr.params.len()
    }

    fn call(&self, interpreter: &Interpreter, arguments: &[Value], _env: Env) -> CallResult {
        let env = BareEnv::for_fn(clone_env(&self.closure)).wrapped();

        for (i, p) in self.fun_expr.params.iter().enumerate() {
            env.borrow_mut()
                .define(p.lexeme.clone(), arguments[i].clone());
        }

        interpreter.execute_block(&self.fun_expr.body, clone_env(&env))?;

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
        if cfg!(feature = "lambda") {
            write!(f, "<fn {}", self.name().unwrap_or("lambda"))
        } else {
            write!(
                f,
                "<fn {}>",
                self.name().expect("A regular function always has a name")
            )
        }
    }
}
