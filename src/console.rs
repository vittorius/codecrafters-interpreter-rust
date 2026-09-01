use std::io;

use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

pub struct RawModeGuard;

impl RawModeGuard {
    pub fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        Ok(RawModeGuard)
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode(); // can't propagate errors from Drop
    }
}