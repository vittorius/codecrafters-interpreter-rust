use std::process::Output;

use crate::common::{TempDir, run_subcommand_with_stdin};

mod common;

/// The REPL reads its own lines from stdin, so a piped script of input drives it
/// exactly like a person typing: every line is read, evaluated, and — with
/// rustyline's non-TTY fast path — no prompts or highlighting are written to
/// stdout, so the program's own output is all a test has to assert on.
fn run_repl(stdin: &str) -> Output {
    // The REPL loads and saves `history.txt` in the current directory, so it
    // runs in a throwaway one to keep the repo clean.
    let dir = TempDir::new();

    run_subcommand_with_stdin("repl", &dir.path, stdin)
}

#[test]
fn test_quit_command_exits_successfully() {
    let output = run_repl(":quit\n");

    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Exiting. Bye-bye."),
        "unexpected stdout:\n{stdout}"
    );
}

#[test]
fn test_end_of_input_exits_successfully() {
    // A closed stdin (no `:quit`) is Ctrl+D: the REPL should still exit 0.
    let output = run_repl("");

    assert_eq!(output.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&output.stdout).contains("Exiting. Bye-bye."));
}

/// The REPL is driven here with a piped stdin, but rustyline still applies the
/// `Validator`, so several lines are joined into one program until the fragment
/// is complete — the same way a person typing them at the prompt would.
///
/// Each submission is then resolved on its own. With the `err-unused-vars`
/// feature that makes a binding declared in one submission and only *used* in a
/// later one fail as an unused variable, which is a pre-existing limitation of
/// that feature in a REPL and unrelated to `readline()`; the tests that span
/// submissions are therefore gated off under it.
#[cfg(not(feature = "err-unused-vars"))]
#[test]
fn test_repl_keeps_state_between_submissions() {
    // The function is declared in one submission and called in the next.
    let output = run_repl("fun f() { return 40 + 2; }\nprint(f() + 1);\n:quit\n");

    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("43"), "unexpected stdout:\n{stdout}");
}

#[cfg(not(feature = "err-unused-vars"))]
#[test]
fn test_repl_joins_multiline_input_into_one_submission() {
    // An unbalanced `{` makes the validator collect lines until it closes;
    // the whole function is then defined in a single resolution pass, so a call
    // on the next line resolves rather than reporting an undefined variable.
    let output = run_repl("fun f() {\n  return 1;\n}\nprint(f());\n:quit\n");

    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains('1'), "unexpected stdout:\n{stdout}");
    assert!(!String::from_utf8_lossy(&output.stderr).contains("Undefined"));
}

#[test]
fn test_repl_reports_runtime_errors_and_keeps_going() {
    let output = run_repl("print(undefinedVar);\nprint \"still alive\";\n:quit\n");

    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Undefined variable"), "got:\n{stderr}");
    assert!(
        stdout.contains("still alive"),
        "unexpected stdout:\n{stdout}"
    );
}

#[cfg(feature = "readline")]
mod readline_tests {
    use super::*;

    #[test]
    fn test_readline_in_the_repl_reads_the_next_input_line() {
        // The REPL consumes the first line as code, `readline()` then blocks on
        // stdin for "hello", and the REPL resumes with ":quit".
        let output = run_repl("print(readline());\nhello\n:quit\n");

        assert_eq!(output.status.code(), Some(0));

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("hello"), "unexpected stdout:\n{stdout}");
        assert!(stdout.contains("Exiting. Bye-bye."));
    }

    #[test]
    fn test_readline_in_the_repl_works_inside_a_function() {
        // Declared and called in one submission, so the test holds whether or
        // not `err-unused-vars` is on.
        let output = run_repl("fun f() { return readline(); } print(f());\nfrom-readline\n:quit\n");

        assert_eq!(output.status.code(), Some(0));

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("from-readline"),
            "unexpected stdout:\n{stdout}"
        );
    }

    #[test]
    fn test_readline_in_the_repl_reads_repeatedly() {
        let output = run_repl("print(readline() + readline());\nfoo\nbar\n:quit\n");

        assert_eq!(output.status.code(), Some(0));

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("foobar"), "unexpected stdout:\n{stdout}");
    }
}
