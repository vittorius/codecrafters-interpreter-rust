use std::fmt::{Debug, Display};

use crate::{environment::Env, interpreter::Interpreter, value::Value};

pub trait Callable: Debug + Display {
    fn arity(&self) -> usize;
    fn call(&self, interpreter: &Interpreter, arguments: &[Value], env: Env) -> Value;
}

// pub enum CallableValue {
//     Function,
//     Class,
// }
