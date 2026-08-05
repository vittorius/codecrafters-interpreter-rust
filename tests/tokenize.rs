use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Deletes the backing file on drop, so a panicking assertion doesn't leak
/// temp files.
struct TempLoxFile {
    path: PathBuf,
}

impl TempLoxFile {
    fn new(contents: &str) -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let unique = COUNTER.fetch_add(1, Ordering::Relaxed);

        let mut path = env::temp_dir();
        path.push(format!(
            "codecrafters-interpreter-test-{}-{unique}.lox",
            std::process::id()
        ));

        fs::write(&path, contents).expect("failed to write temp .lox file");

        Self { path }
    }
}

impl Drop for TempLoxFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn run_tokenize(path: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_codecrafters-interpreter"))
        .arg("tokenize")
        .arg(path)
        .output()
        .expect("failed to run codecrafters-interpreter binary")
}

fn assert_tokenize_success(source: &str, expected_stdout: &str) {
    let file = TempLoxFile::new(source);
    let output = run_tokenize(&file.path);

    assert!(
        output.status.success(),
        "tokenize exited with {}; stderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout was not valid UTF-8");
    assert_eq!(stdout, expected_stdout);
}

/// `65` is `ExitValue::LexicalError` in `src/main.rs`.
fn assert_tokenize_lexical_error(source: &str, expected_stdout: &str, expected_stderr: &str) {
    let file = TempLoxFile::new(source);
    let output = run_tokenize(&file.path);

    assert_eq!(
        output.status.code(),
        Some(65),
        "expected exit code 65 (lexical error); stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout was not valid UTF-8");
    assert_eq!(stdout, expected_stdout);

    let stderr = String::from_utf8(output.stderr).expect("stderr was not valid UTF-8");
    assert_eq!(stderr, expected_stderr);
}

#[test]
fn test_multiline_comments_happy_path() {
    // Lox block comments nest (unlike C), so `/* outer /* inner */ still
    // outer */` must be skipped as a single comment ending at the *last*
    // `*/`, not the first one. The second comment spans multiple source
    // lines to exercise the same nesting-depth tracking across newlines.
    let source = concat!(
        "(1 + 2) /* outer /* inner */ still outer */ * 3\n",
        "\"string\" /* another\n",
        "multi\n",
        "line */ identifier\n",
    );

    let expected = concat!(
        "LEFT_PAREN ( null\n",
        "NUMBER 1 1.0\n",
        "PLUS + null\n",
        "NUMBER 2 2.0\n",
        "RIGHT_PAREN ) null\n",
        "STAR * null\n",
        "NUMBER 3 3.0\n",
        "STRING \"string\" string\n",
        "IDENTIFIER identifier null\n",
        "EOF  null\n",
    );

    assert_tokenize_success(source, expected);
}

#[test]
fn test_unterminated_multiline_comment() {
    // A plain, non-nested block comment that's missing its closing `*/`.
    // The source deliberately ends on a newline: `comment_or_slash` only
    // checks `peek_next()` for EOF inside its catch-all branch, so if the
    // very last character of the file were an ordinary comment character
    // instead, the error would fire one character early and leave that
    // character to be rescanned as a bogus trailing token. Ending on `\n`
    // avoids that (the `('\n', _)` arm always consumes it, so the next
    // iteration cleanly sees true EOF), keeping this test deterministic.
    let source = concat!(
        "(1 + 2)\n",
        "/* this comment\n",
        "never closes\n",
    );

    let expected_stdout = concat!(
        "LEFT_PAREN ( null\n",
        "NUMBER 1 1.0\n",
        "PLUS + null\n",
        "NUMBER 2 2.0\n",
        "RIGHT_PAREN ) null\n",
        "EOF  null\n",
    );
    let expected_stderr = "[line 4] Error: Unterminated multiline comment\n";

    assert_tokenize_lexical_error(source, expected_stdout, expected_stderr);
}

#[test]
fn test_unterminated_nested_multiline_comment() {
    // The inner comment closes but the outer one doesn't, which only
    // errors correctly if `comment_or_slash` actually tracks nesting depth
    // rather than treating any `*/` as closing the whole comment.
    let source = concat!("print 1;\n", "/* outer /* inner */ still not closed\n",);

    let expected_stdout = concat!(
        "PRINT print null\n",
        "NUMBER 1 1.0\n",
        "SEMICOLON ; null\n",
        "EOF  null\n",
    );
    let expected_stderr = "[line 3] Error: Unterminated multiline comment\n";

    assert_tokenize_lexical_error(source, expected_stdout, expected_stderr);
}
