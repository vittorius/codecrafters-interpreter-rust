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

#[test]
#[ignore = "Scanner does not yet support nested multiline comments — see TODO in Scanner::comment_or_slash"]
fn test_multiline_comments() {
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

    let file = TempLoxFile::new(source);
    let output = run_tokenize(&file.path);

    assert!(
        output.status.success(),
        "tokenize exited with {}; stderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout was not valid UTF-8");
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

    assert_eq!(stdout, expected);
}
