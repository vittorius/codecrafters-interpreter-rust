#![allow(dead_code)]

use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Deletes the backing file on drop, so a panicking assertion doesn't leak
/// temp files.
pub struct TempLoxFile {
    pub path: PathBuf,
}

impl TempLoxFile {
    pub fn new(contents: &str) -> Self {
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

pub fn run_binary(command: &str, path: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_codecrafters-interpreter"))
        .arg(command)
        .arg(path)
        .output()
        .expect("failed to run codecrafters-interpreter binary")
}

/// Runs the binary with `stdin` piped, feeding it `stdin_contents`.
///
/// `Command::output()` (used by [`run_binary`]) nulls stdin, so this is the only
/// way to test input-consuming code paths such as the `readline()` native fn.
/// Writing before waiting is safe here because the payloads are tiny and the
/// pipe buffer holds them.
pub fn run_binary_with_stdin(command: &str, path: &Path, stdin_contents: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_codecrafters-interpreter"))
        .arg(command)
        .arg(path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn codecrafters-interpreter binary");

    child
        .stdin
        .take()
        .expect("child stdin was not piped")
        .write_all(stdin_contents.as_bytes())
        .expect("failed to write to the child's stdin");

    child
        .wait_with_output()
        .expect("failed to wait for the child")
}

/// Runs the binary with `subcommand` as its only argument and `stdin` piped.
///
/// Used for `repl`, which takes no file argument. `current_dir` is the working
/// directory for the child, so tests can keep the REPL's `history.txt` out of
/// the repo.
pub fn run_subcommand_with_stdin(
    subcommand: &str,
    current_dir: &Path,
    stdin_contents: &str,
) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_codecrafters-interpreter"))
        .arg(subcommand)
        .current_dir(current_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn codecrafters-interpreter binary");

    child
        .stdin
        .take()
        .expect("child stdin was not piped")
        .write_all(stdin_contents.as_bytes())
        .expect("failed to write to the child's stdin");

    child
        .wait_with_output()
        .expect("failed to wait for the child")
}

/// A temp directory, removed on drop, so tests that need a working directory
/// (the REPL writes `history.txt` into the current one) leave nothing behind.
pub struct TempDir {
    pub path: PathBuf,
}

impl TempDir {
    pub fn new() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let unique = COUNTER.fetch_add(1, Ordering::Relaxed);

        let mut path = env::temp_dir();
        path.push(format!(
            "codecrafters-interpreter-test-{}-{unique}",
            std::process::id()
        ));

        fs::create_dir_all(&path).expect("failed to create temp dir");

        Self { path }
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
