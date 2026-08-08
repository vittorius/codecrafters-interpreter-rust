pub fn error(line: usize, message: &str) {
    error_at(line, "", message);
}

pub fn error_at(line: usize, at: &str, message: &str) {
    eprintln!("[line {line}] Error{at}: {message}");
}
