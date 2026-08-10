#[deprecated]
pub fn error(line: usize, message: &str) {
    error_at(line, "", message);
}

#[deprecated]
pub fn error_at(line: usize, at: &str, message: &str) {
    eprintln!("{}", fmt_error_at(line, at, message));
}

pub fn fmt_error(line: usize, message: &str) -> String {
    fmt_error_at(line, "", message)
}

pub fn fmt_error_at(line: usize, at: &str, message: &str) -> String {
    format!("[line {line}] Error{at}: {message}")
}