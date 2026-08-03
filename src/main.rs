#![allow(unused_variables)]
use std::env;
use std::fs;

use crate::scanner::Scanner;

mod scanner;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} tokenize <filename>", args[0]);
        return;
    }

    let command = &args[1];
    let filename = &args[2];

    match command.as_str() {
        "tokenize" => {
            // You can use print statements as follows for debugging, they'll be visible when running tests.
            eprintln!("Logs from your program will appear here!");

            let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
                eprintln!("Failed to read file {}", filename);
                String::new()
            });

            let mut scanner = Scanner::new(file_contents);

            for token in scanner.scan_tokens() {
                println!("{token}");
            }
        }
        _ => {
            eprintln!("Unknown command: {}", command);
        }
    }
}
