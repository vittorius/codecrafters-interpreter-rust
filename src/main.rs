#![allow(unused_variables)]
use std::env;
use std::fs;
use std::process::ExitCode;

use crate::scanner::Scanner;

mod scanner;

#[repr(u8)]
enum ExitValue {
    Success = 0,
    Usage = 64,
    LexicalError = 65,
}

impl From<ExitValue> for ExitCode {
    fn from(value: ExitValue) -> Self {
        Self::from(value as u8)
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} tokenize <filename>", args[0]);
        return ExitValue::Usage.into();
    }

    let command = &args[1];
    let filename = &args[2];

    match command.as_str() {
        "tokenize" => {
            // You can use print statements as follows for debugging, they'll be visible when running tests.
            // eprintln!("Logs from your program will appear here!");

            let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
                eprintln!("Failed to read file {}", filename);
                String::new()
            });

            let mut scanner = Scanner::new(file_contents);
            scanner.scan_tokens();
            
            for token in scanner.tokens() {
                println!("{token}");
            }

            if scanner.has_error {
                ExitValue::LexicalError.into()
            } else {
                ExitValue::Success.into()
            }
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            ExitValue::Usage.into()
        }
    }
}
