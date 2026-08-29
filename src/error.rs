use crate::{lox, scanner::Token};

#[derive(Debug)]
pub struct RuntimeError(String);

impl RuntimeError {
    pub fn new(token: &Token<'_>, message: &str) -> Self {
        RuntimeError(lox::fmt_runtime_error(token.line, message))
    }
}

pub struct ErrorSink {
    errors: Vec<String>,
}

impl ErrorSink {
    pub fn new() -> Self {
        Self { errors: vec![] }
    }

    pub fn errors(&self) -> impl Iterator<Item = &str> {
        self.errors.iter().map(|s| s.as_str())
    }

    pub fn append(&mut self, msg: &str) {
        self.errors.push(msg.to_owned());
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }
}
