// TODO: move these functions to error module and delete this module

pub fn fmt_error(line: usize, message: &str) -> String {
    fmt_error_at(line, "", message)
}

pub fn fmt_error_at(line: usize, at: &str, message: &str) -> String {
    format!("[line {line}] Error{at}: {message}")
}

pub fn fmt_runtime_error(line: usize, message: &str) -> String {
    format!("{message}\n[line {line}]")
}
