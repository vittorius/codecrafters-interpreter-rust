// TODO: move inside the 'interpreter' module

use std::{
    fmt::{Debug, Display},
    rc::Rc,
};

use crate::{error::RuntimeError, interpreter::Interpreter, value::Value};

pub type CallResult = Result<Value, RuntimeError>;

pub trait Callable: Debug + Display {
    fn arity(&self) -> usize;
    fn call(self: Rc<Self>, interpreter: &Interpreter, arguments: &[Value]) -> CallResult;
}

pub trait SharedClone {
    fn shared_clone(self: &Rc<Self>) -> Rc<Self> {
        Rc::clone(self)
    }
}
