use crate::common::{TempLoxFile, run_binary};

mod common;

#[allow(dead_code)]
fn run_program(path: &std::path::Path) -> std::process::Output {
    run_binary("run", path)
}

#[allow(dead_code)]
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

#[allow(dead_code)]
fn assert_run_parse_error(source: &str, expected_stderr_substring: &str) {
    let file = TempLoxFile::new(source);
    let output = run_program(&file.path);

    assert_eq!(output.status.code(), Some(65));

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(expected_stderr_substring),
        "expected stderr to contain {:?}, got:\n{}",
        expected_stderr_substring,
        stderr
    );
}

#[allow(dead_code)]
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

#[cfg(feature = "init-vars")]
mod init_vars_tests {
    use super::*;

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

#[cfg(feature = "lambdas")]
mod lambda_tests {
    use super::*;

    #[test]
    fn test_pass_lambda_into_function() {
        assert_run_success(
            r#"
           fun thrice(fn) {
             for (var i = 1; i <= 3; i = i + 1) {
               fn(i);
             }
           }

           thrice(fun (a) {
             print a;
           });
           "#,
            "1\n2\n3\n",
        );
    }

    #[test]
    fn test_assign_lambda_to_variable() {
        assert_run_success(
            r#"
                var f = fun (a) {
                  print a;
                };

                f(10);
               "#,
            "10\n",
        );
    }

    #[test]
    fn test_lambda_in_expression_statement_does_nothing() {
        assert_run_parse_error(
            "fun () {};",
            "[line 1] Error at 'fun': Lambda functions are forbidden in statements",
        );
    }
}
