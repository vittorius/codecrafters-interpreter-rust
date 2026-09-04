use std::path::Path;
use std::process::Output;

mod common;

use common::{TempLoxFile, run_binary};

#[allow(dead_code)]
fn run_evaluate(path: &Path) -> Output {
    run_binary("evaluate", path)
}

#[allow(dead_code)]
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

#[cfg(feature = "comma-op")]
mod comma_op_tests {
    use crate::assert_evaluate_success;
    use crate::common::TempLoxFile;
    use crate::run_evaluate;

    #[test]
    fn test_comma_evaluates_left_discards_it_and_returns_right() {
        assert_evaluate_success("1 + 2, 40 + 2", "42\n");

        let file = TempLoxFile::new("undefined, 42");
        let output = run_evaluate(&file.path);

        assert_eq!(output.status.code(), Some(70));
    }
}

#[cfg(feature = "str-cmp")]
mod str_cmp_tests {
    use crate::assert_evaluate_success;

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
}

#[cfg(feature = "str-num-concat")]
mod str_num_concat_tests {
    use crate::assert_evaluate_success;

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
}

#[cfg(feature = "init-vars")]
mod init_vars_tests {
    use crate::common::{TempLoxFile, run_binary};

    fn run_program(path: &std::path::Path) -> std::process::Output {
        run_binary("run", path)
    }

    fn assert_run_success(source: &str, expected_stdout: &str) {
        let file = TempLoxFile::new(source);
        let output = run_program(&file.path);

        assert!(
            output.status.success(),
            "run exited with {}; stderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );

        let stdout = String::from_utf8(output.stdout).expect("stdout was not valid UTF-8");
        assert_eq!(stdout, expected_stdout);
    }

    fn assert_run_runtime_error(source: &str, expected_stderr_substring: &str) {
        let file = TempLoxFile::new(source);
        let output = run_program(&file.path);

        assert_eq!(output.status.code(), Some(70));

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains(expected_stderr_substring),
            "expected stderr to contain {:?}, got:\n{}",
            expected_stderr_substring,
            stderr
        );
    }

    #[test]
    fn test_uninitialized_variable_is_runtime_error() {
        assert_run_runtime_error("var x; print x;", "Uninitialized variable");
    }

    #[test]
    fn test_initialized_variable_returns_its_value() {
        assert_run_success("var x = 42; print x;", "42\n");
    }

    #[test]
    fn test_explicit_nil_initializer_is_also_uninitialized() {
        // `nil` can't be distinguished from "no initializer" at the storage
        // level (both store Value::Nil), so init-vars flags both the same way.
        assert_run_runtime_error("var x = nil; print x;", "Uninitialized variable");
    }

    #[test]
    fn test_never_declared_variable_is_undefined_not_uninitialized() {
        assert_run_runtime_error("x;", "Undefined variable");
    }
}
