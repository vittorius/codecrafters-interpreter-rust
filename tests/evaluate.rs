use std::path::Path;
use std::process::Output;

mod common;

use common::{run_binary, TempLoxFile};

fn run_evaluate(path: &Path) -> Output {
    run_binary("evaluate", path)
}

fn assert_evaluate_success(source: &str, expected_stdout: &str) {
    let file = TempLoxFile::new(source);
    let output = run_evaluate(&file.path);

    assert!(
        output.status.success(),
        "evaluate exited with {}; stderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout was not valid UTF-8");
    assert_eq!(stdout, expected_stdout);
}

#[test]
fn test_string_greater_than_true() {
    assert_evaluate_success(r#""banana" > "apple""#, "true\n");
}

#[test]
fn test_string_greater_than_false() {
    assert_evaluate_success(r#""apple" > "banana""#, "false\n");
}

#[test]
fn test_string_less_than_true() {
    assert_evaluate_success(r#""apple" < "banana""#, "true\n");
}

#[test]
fn test_string_less_than_false() {
    assert_evaluate_success(r#""banana" < "apple""#, "false\n");
}

#[test]
fn test_string_greater_equal_true_for_equal_strings() {
    assert_evaluate_success(r#""apple" >= "apple""#, "true\n");
}

#[test]
fn test_string_greater_equal_false() {
    assert_evaluate_success(r#""apple" >= "banana""#, "false\n");
}

#[test]
fn test_string_less_equal_true_for_equal_strings() {
    assert_evaluate_success(r#""apple" <= "apple""#, "true\n");
}

#[test]
fn test_string_less_equal_false() {
    assert_evaluate_success(r#""banana" <= "apple""#, "false\n");
}

#[test]
fn test_string_comparison_is_lexicographic_not_length_based() {
    // A longer string can still be "less than" a shorter one: `'z'` (0x7A)
    // sorts after `'a'` (0x61) at the first character, regardless of length.
    assert_evaluate_success(r#""z" < "aa""#, "false\n");
}

#[test]
fn test_string_comparison_prefix_is_less_than_extended_string() {
    // A proper prefix sorts before the longer string it prefixes.
    assert_evaluate_success(r#""app" < "apple""#, "true\n");
}

#[test]
fn test_concat_string_left_number_right() {
    assert_evaluate_success(r#""value: " + 1"#, "value: 1\n");
}

#[test]
fn test_concat_number_left_string_right() {
    assert_evaluate_success(r#"1 + " item""#, "1 item\n");
}

#[test]
fn test_concat_string_left_decimal_number_right() {
    assert_evaluate_success(r#""pi is " + 3.14"#, "pi is 3.14\n");
}

#[test]
fn test_concat_decimal_number_left_string_right() {
    assert_evaluate_success(r#"3.14 + " is pi""#, "3.14 is pi\n");
}
