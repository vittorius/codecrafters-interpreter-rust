use std::{fmt::Display, rc::Rc};

use crate::{
    callable::Callable, callable::SharedClone, class::ClassShared, function::FunctionShared,
    instance::InstanceShared, native_function::NativeFunctionShared,
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
    // str is a value object we "copy" the entire string whey "copying" a value (actually, .clone())
    Str(String),
    Num(f64),
    Bool(bool),
    // these *Shared types are "ref" objects with all shallow copies
    // pointing to the same object in memory
    NativeFn(NativeFunctionShared),
    Fn(FunctionShared),
    Class(ClassShared),
    Object(InstanceShared),
    Nil,
}

impl Value {
    pub fn as_callable(&self) -> Option<Rc<dyn Callable>> {
        match self {
            Value::NativeFn(native_fn) => Some(native_fn.shared_clone()),
            Value::Fn(function) => Some(function.shared_clone()),
            Value::Class(class) => Some(class.shared_clone()),
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
            Value::NativeFn(native_fn) => write!(f, "{native_fn}"),
            Value::Fn(function) => write!(f, "{function}"),
            Value::Class(class) => write!(f, "{class}"),
            Value::Object(instance) => write!(f, "{}", instance.borrow()),
            Value::Nil => write!(f, "nil"),
        }
    }
}
