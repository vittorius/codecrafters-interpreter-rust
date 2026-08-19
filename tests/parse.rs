use std::path::Path;
use std::process::Output;

mod common;

use common::{run_binary, TempLoxFile};

fn run_parse(path: &Path) -> Output {
    run_binary("parse", path)
}

fn assert_parse_success(source: &str, expected_stdout: &str) {
    let file = TempLoxFile::new(source);
    let output = run_parse(&file.path);

    assert!(
        output.status.success(),
        "parse exited with {}; stderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout was not valid UTF-8");
    assert_eq!(stdout, expected_stdout);
}

/// `65` is `ExitValue::SyntaxError` in `src/main.rs`. As of now, `main.rs`
/// discards the `ParseError` message on failure rather than printing it, so
/// both stdout and stderr are expected to be empty here -- that's today's
/// actual behavior, not a claim that it's the desired end state.
fn assert_parse_syntax_error(source: &str) {
    let file = TempLoxFile::new(source);
    let output = run_parse(&file.path);

    assert_eq!(
        output.status.code(),
        Some(65),
        "expected exit code 65 (syntax error); stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout was not valid UTF-8");
    assert_eq!(stdout, "");

    let stderr = String::from_utf8(output.stderr).expect("stderr was not valid UTF-8");
    assert_eq!(stderr, "");
}

#[test]
fn test_comma_two_operands() {
    assert_parse_success("1, 2", "(, 1.0 2.0)\n");
}

#[test]
fn test_comma_is_left_associative() {
    // `expression`'s `while` loop must nest leftward: `1, 2, 3` parses as
    // `(1, 2), 3`, not `1, (2, 3)`.
    assert_parse_success("1, 2, 3", "(, (, 1.0 2.0) 3.0)\n");
}

#[test]
fn test_comma_has_lower_precedence_than_equality() {
    // `compound`'s operands are `equality` expressions, so `==` binds
    // tighter than `,`.
    assert_parse_success("1 == 1, 2 == 2", "(, (== 1.0 1.0) (== 2.0 2.0))\n");
}

#[test]
fn test_comma_trailing_operator_is_syntax_error() {
    // After consuming the `,`, `equality()` fails to find a right operand.
    assert_parse_syntax_error("1,");
}
