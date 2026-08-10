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
