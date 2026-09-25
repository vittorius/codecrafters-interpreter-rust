// TODO: move inside the 'interpreter' module

use std::{fmt::Display, rc::Rc};

use crate::{
    callable::{CallResult, Callable},
    environment::Env,
    expr::fun_expr::FunExpr,
    instance::InstanceShared,
    interpreter::Interpreter,
    stmt::fun_decl::FunDecl,
    value::Value,
};

#[derive(Debug)]
enum FunDef {
    Function(Rc<FunDecl>), // all functions of the same definition will share the same declaration
    #[cfg(feature = "lambdas")]
    Lambda(FunExpr), // lambda declarations are never cloned (in particular, because they are never bound)
}

#[derive(Debug)]
pub struct Function {
    definition: FunDef,
    closure: Env,
    is_initializer: bool,
}

pub type FunctionShared = Rc<Function>;

impl Function {
    pub fn new(decl: Rc<FunDecl>, closure: Env, is_initializer: bool) -> Self {
        Self {
            definition: FunDef::Function(decl),
            closure,
            is_initializer,
        }
    }

    #[cfg(feature = "lambdas")]
    pub fn new_lambda(fun_expr: FunExpr, closure: Env) -> Self {
        Self {
            definition: FunDef::Lambda(fun_expr),
            closure,
            is_initializer: false,
        }
    }

    pub fn name(&self) -> &str {
        match &self.definition {
            FunDef::Function(fun_decl) => &fun_decl.name.lexeme,
            #[cfg(feature = "lambdas")]
            FunDef::Lambda(_) => "(lambda)",
        }
    }

    pub fn bind(&self, instance: InstanceShared) -> FunctionShared {
        let env = Env::new(Some(&self.closure));
        env.define("this".to_owned(), Value::Object(instance));

        #[cfg_attr(not(feature = "lambdas"), allow(irrefutable_let_patterns))]
        let FunDef::Function(fun_decl) = &self.definition else {
            unreachable!("Only functions are bound to class instances")
        };

        Rc::new(Function::new(Rc::clone(fun_decl), env, self.is_initializer))
    }

    fn this_in_initializer(&self) -> Value {
        self.closure
            .get_at(0, "this")
            .expect("'this' is always defined if a function is a class initializer")
    }

    fn fun_expr(&self) -> &FunExpr {
        match &self.definition {
            FunDef::Function(fun_decl) => &fun_decl.expr,
            #[cfg(feature = "lambdas")]
            FunDef::Lambda(fun_expr) => fun_expr,
        }
    }
}

impl Callable for Function {
    fn arity(&self) -> usize {
        self.fun_expr().params.len()
    }

    // TODO: rethink Rc<Self> as a receiver type
    fn call(self: Rc<Self>, interpreter: &Interpreter, arguments: &[Value]) -> CallResult {
        let env = Env::new(Some(&self.closure));

        self.fun_expr()
            .params
            .iter()
            .zip(arguments)
            .for_each(|(param, arg)| {
                env.define(param.lexeme.clone(), arg.clone());
            });

        interpreter.execute_block(&self.fun_expr().body, &env)?;

        if self.is_initializer {
            // always return 'this' from an initializer
            Ok(self.this_in_initializer())
        } else if let Some(return_value) = env.take_return_from_fn() {
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
        write!(f, "<fn {}>", self.name())
    }
}
