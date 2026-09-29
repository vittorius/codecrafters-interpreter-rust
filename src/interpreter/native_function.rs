use std::{
    fmt::{Debug, Display},
    rc::Rc,
};

use crate::{
    interpreter::Interpreter,
    interpreter::callable::{CallResult, Callable, CloneRef},
    interpreter::value::Value,
};

pub struct NativeFunction {
    arity: usize,
    callback: Box<dyn Fn(Vec<Value>) -> CallResult>,
}

pub type NativeFunctionRef = Rc<NativeFunction>;

impl NativeFunction {
    pub fn new_ref<F>(arity: usize, callback: F) -> NativeFunctionRef
    where
        F: Fn(Vec<Value>) -> CallResult + 'static,
    {
        Rc::new(Self {
            arity,
            callback: Box::new(callback),
        })
    }
}

impl Callable for NativeFunction {
    fn arity(&self) -> usize {
        self.arity
    }

    fn call(self: Rc<Self>, _interpreter: &Interpreter, arguments: &[Value]) -> CallResult {
        (self.callback)(Vec::from(arguments))
    }
}

impl CloneRef for NativeFunction {}

impl Debug for NativeFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native fn>")
    }
}

impl Display for NativeFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native fn>")
    }
}
