use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
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
