use std::fmt::{Debug, Display};

use crate::{environment::Env, error::RuntimeError, interpreter::Interpreter, value::Value};

pub type CallResult = Result<Value, RuntimeError>;

pub trait Callable: Debug + Display {
    fn arity(&self) -> usize;
    fn call(&self, interpreter: &Interpreter, arguments: &[Value], env: Env) -> CallResult;
}
