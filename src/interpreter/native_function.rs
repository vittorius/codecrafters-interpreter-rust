use std::{fmt::Display, rc::Rc};

use crate::{
    interpreter::Interpreter,
    interpreter::callable::{CallResult, Callable, CloneRef},
    interpreter::value::Value,
};

#[derive(Debug)]
pub struct NativeFunction {
    callback: fn() -> CallResult,
}

pub type NativeFunctionRef = Rc<NativeFunction>;

impl NativeFunction {
    pub fn new_ref(callback: fn() -> CallResult) -> NativeFunctionRef {
        Rc::new(Self { callback })
    }
}

impl Callable for NativeFunction {
    fn arity(&self) -> usize {
        0
    }

    fn call(self: Rc<Self>, _interpreter: &Interpreter, _arguments: &[Value]) -> CallResult {
        (self.callback)()
    }
}

impl CloneRef for NativeFunction {}

impl Display for NativeFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native fn>")
    }
}
