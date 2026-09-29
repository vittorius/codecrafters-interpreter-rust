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

#[cfg(test)]
mod tests {
    use super::*;

    /// Calls a native fn of the given arity with the given arguments, the way
    /// `visit_call_expr` does after its arity check: it forwards whatever
    /// survived the check to the callback.
    fn call(
        arity: usize,
        callback: impl Fn(Vec<Value>) -> CallResult + 'static,
        arguments: &[Value],
    ) -> CallResult {
        let native = NativeFunction::new_ref(arity, callback);
        assert_eq!(
            arguments.len(),
            native.arity(),
            "arity mismatch in the test"
        );

        let interpreter = Interpreter::new();
        native.call(&interpreter, arguments)
    }

    /// A native fn of a given arity, for the tests that only inspect metadata.
    fn native(arity: usize) -> NativeFunctionRef {
        NativeFunction::new_ref(arity, |_| Ok(Value::Nil))
    }

    #[test]
    fn arity_is_reported() {
        assert_eq!(native(2).arity(), 2);
    }

    #[test]
    fn callback_receives_the_arguments() {
        // The second argument proves the order is preserved.
        let value = call(
            2,
            |args| Ok(args.into_iter().nth(1).unwrap_or(Value::Nil)),
            &[Value::Str("first".into()), Value::Str("second".into())],
        )
        .expect("native fn should succeed");

        assert!(matches!(value, Value::Str(s) if s == "second"));
    }

    #[test]
    fn a_zero_arity_callback_gets_no_arguments() {
        let value = call(
            0,
            |args| Ok(Value::Str(format!("{} args", args.len()))),
            &[],
        )
        .expect("native fn should succeed");

        assert!(matches!(value, Value::Str(s) if s == "0 args"));
    }

    #[test]
    fn debug_and_display_render_a_native_fn() {
        let native = native(0);

        assert_eq!(native.to_string(), "<native fn>");
        assert_eq!(format!("{native:?}"), "<native fn>");
    }
}
