use std::{
    fmt::Display,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    callable::{CallResult, Callable},
    environment::Env,
    interpreter::Interpreter,
    value::Value,
};

#[derive(Debug)]
pub struct ClockFunction;

impl Callable for ClockFunction {
    fn arity(&self) -> usize {
        0
    }

    fn call(&self, interpreter: &Interpreter, arguments: &[Value], env: Env) -> CallResult {
        Ok(Value::Num(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time is before the Unix epoch")
                .as_secs_f64(),
        ))
    }
}

impl Display for ClockFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native fn>")
    }
}
