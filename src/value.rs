use std::{cell::RefCell, fmt::Display, rc::Rc};

use crate::{callable::Callable, instance::Instance};

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
    // callable is a ref object, we don't "copy" callables in runtime
    Callable(Rc<dyn Callable>),
    // class instance is a ref object, we "copy" only a reference to it;
    // we need RefCell because it's a mutable bag of properties
    Object(Rc<RefCell<Instance>>),
    Nil,
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Str(value) => write!(f, "{value}"),
            Value::Num(value) => write!(f, "{value}"),
            Value::Bool(value) => write!(f, "{value}"),
            Value::Callable(callable) => write!(f, "{callable}"),
            Value::Object(instance) => write!(f, "{}", instance.borrow()),
            Value::Nil => write!(f, "nil"),
        }
    }
}
