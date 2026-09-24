// TODO: move inside the 'interpreter' module

use std::{fmt::Display, rc::Rc};

use crate::{
    callable::{CallResult, Callable},
    environment::{Env, EnvShared, clone_env},
    expr::fun_expr::FunExpr,
    instance::InstanceShared,
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
    name: Option<Token>, // optional because it may be a lambda
    fun_expr: FunExpr,
    closure: EnvShared,
    is_initializer: bool,
}

// TODO: type FunctionShared

impl Function {
    // FIXME: store shared FunDecl in the Function
    pub fn new(decl: FunDecl, closure: EnvShared, is_initializer: bool) -> Self {
        Self {
            name: Some(decl.name),
            fun_expr: decl.expr,
            closure,
            is_initializer,
        }
    }

    #[cfg(feature = "lambdas")]
    pub fn new_lambda(fun_expr: FunExpr, closure: EnvShared) -> Self {
        Self {
            name: None,
            fun_expr,
            closure,
            is_initializer: false,
        }
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(|t| t.lexeme.as_str())
    }

    // TODO: return FunctionShared; or think if we need it here
    // or the Function-using code is better to decide whether to wrap in Rc
    // pub fn into_shared(self) -> Rc<Self> {
    //    Rc::new()
    // }

    pub fn bind(&self, instance: InstanceShared) -> Self {
        let mut env = Env::with_enclosing(clone_env(&self.closure));
        env.define("this".to_owned(), Value::Object(instance));
        // FIXME: reject cloning in favor of shared function declarations that must be kept shared in runtime
        Function::new(
            FunDecl {
                name: self
                    .name
                    .as_ref()
                    .cloned()
                    .expect("Regular function always has a name"),
                expr: self.fun_expr.clone(),
            },
            env.wrapped(),
            self.is_initializer,
        )
    }

    fn this_in_initializer(&self) -> Value {
        self.closure
            .borrow()
            .get_at(0, "this")
            .expect("'this' is always defined if a function is a class initializer")
    }
}

impl Callable for Function {
    fn arity(&self) -> usize {
        self.fun_expr.params.len()
    }

    // TODO: rethink Rc<Self> as a receiver type
    fn call(self: Rc<Self>, interpreter: &Interpreter, arguments: &[Value]) -> CallResult {
        let env = Env::with_enclosing(clone_env(&self.closure)).wrapped();

        for (i, p) in self.fun_expr.params.iter().enumerate() {
            env.borrow_mut()
                .define(p.lexeme.clone(), arguments[i].clone());
        }

        interpreter.execute_block(&self.fun_expr.body, clone_env(&env))?;

        if self.is_initializer {
            // always return 'this' from an initializer
            Ok(self.this_in_initializer())
        } else if let Some(return_value) = env.borrow_mut().clear_return_from_fn() {
            // The interpreter stack was naturally unwinded by the early return in the Interpreter::execute
            // and there was an actual return value stored in the env.
            // Return the `return` value and clear the "returning" env state.

            Ok(return_value.clone())
        } else {
            Ok(Value::Nil)
        }
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if cfg!(feature = "lambdas") {
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
