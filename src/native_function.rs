use std::{fmt::Display, rc::Rc};

use crate::{
    callable::{CallResult, Callable,  SharedClone},
    interpreter::Interpreter,
    value::Value,
};

#[derive(Debug)]
pub struct NativeFunction {
    callback: fn() -> CallResult,
}

pub type NativeFunctionShared = Rc<NativeFunction>;

impl NativeFunction {
    pub fn new_shared(callback: fn() -> CallResult) -> NativeFunctionShared {
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

impl SharedClone for NativeFunction {}

impl Display for NativeFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native fn>")
    }
}
