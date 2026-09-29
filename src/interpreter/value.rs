use std::{fmt::Display, rc::Rc};

use crate::{
    interpreter::callable::Callable, interpreter::callable::CloneRef, interpreter::class::ClassRef,
    interpreter::function::FunctionRef, interpreter::instance::InstanceRef,
    interpreter::native_function::NativeFunctionRef,
};

// We made Value cloneable because we need to be able to store values in the environment
// and refer to variable in expressions. We construct a new Value in 2 cases: evaluating expressions
// and defining variables. Therefore, we cannot maintain a single place where values are stored.
// Initially, it was because of String values, and we could have a dedicated string interner
// to own strings. String values would be Value::Str(&'v str). But it seems to be an overkill, and we
// just clone Strings (and Values) when we evaluate variables and get their values from the environment.
// The environment owns Values.
// It was decided to encode value/ref separation in Value itself (not in Environment)
// to make Values "copyable" around (via .clone()) everywhere when working with the parse tree.
#[derive(Clone, Debug)]
pub enum Value {
    Str(String),
    Num(f64),
    Bool(bool),
    NativeFn(NativeFunctionRef),
    Fn(FunctionRef),
    Class(ClassRef),
    Object(InstanceRef),
    Nil,
}

impl Value {
    pub fn as_callable(&self) -> Option<Rc<dyn Callable>> {
        match self {
            Value::NativeFn(function) => Some(function.clone_ref()),
            Value::Fn(function) => Some(function.clone_ref()),
            Value::Class(class) => Some(class.clone_ref()),
            _ => None,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Str(value) => write!(f, "{value}"),
            Value::Num(value) => write!(f, "{value}"),
            Value::Bool(value) => write!(f, "{value}"),
            Value::NativeFn(function) => write!(f, "{function}"),
            Value::Fn(function) => write!(f, "{function}"),
            Value::Class(class) => write!(f, "{class}"),
            Value::Object(instance) => write!(f, "{}", instance.borrow()),
            Value::Nil => write!(f, "nil"),
        }
    }
}
