use std::{fmt::Display, rc::Rc};

use crate::callable::Callable;

// TODO: there could be an enum Object { Value, Ref }
// and Ref can hold stings and class objects, others go into Value.
//
// We made Value cloneable because we need to be able to store values in the environment
// and refer to variable in expressions. We construct a new Value in 2 cases: evaluating expressions
// and defining variables. Therefore, we cannot maintain a single place where values are stored.
// Actually, it's mostly because of String values, and we could have a dedicated string interner
// to own strings. String values would be Value::Str(&'v str). But it seems to be an overkill, and we
// just clone Strings (and Values) when we evaluate variables and get their values from the environment.
// The environment owns Values.
#[derive(Clone, Debug)]
pub enum Value {
    Str(String),
    Num(f64),
    Bool(bool),
    Callable(Rc<dyn Callable>), // cloning a callable just returns a pointer to it ("ref object" behavior)
    Nil,
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Str(value) => write!(f, "{value}"),
            Value::Num(value) => write!(f, "{value}"),
            Value::Bool(value) => write!(f, "{value}"),
            Value::Callable(callable) => write!(f, "{callable}"),
            Value::Nil => write!(f, "nil"),
        }
    }
}
