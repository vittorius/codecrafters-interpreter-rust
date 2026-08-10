#![allow(dead_code, unused_variables)]

use std::env;
use std::fs;
use std::process::ExitCode;

use crate::ast_printer::AstPrinter;
use crate::parser::Parser;
use crate::scanner::Scanner;

mod ast_printer;
mod error;
mod expr;
mod lox;
mod parser;
mod rpn_ast_printer;
mod scanner;

#[repr(u8)]
enum ExitValue {
    Success = 0,
    Usage = 64,
    SyntaxError = 65, // lexical or syntactical grammar error
    RuntimeError = 70,
}

impl From<ExitValue> for ExitCode {
    fn from(value: ExitValue) -> Self {
        Self::from(value as u8)
    }
}

fn main() -> ExitCode {
    // memo: print your logs using eprintln!

    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} (tokenize | parse) <filename>", args[0]);
        return ExitValue::Usage.into();
    }

    let command = &args[1];
    let filename = &args[2];

    match command.as_str() {
        "tokenize" => tokenize(filename).into(),
        "parse" => parse(filename).into(),
        _ => {
            eprintln!("Unknown command: {}", command);
            ExitValue::Usage.into()
        }
    }
}

fn tokenize(filename: &str) -> ExitValue {
    let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
        eprintln!("Failed to read file {}", filename);
        String::new()
    });

    let scanner = Scanner::new(&file_contents);
    match scanner.scan_tokens() {
        scanner::Result::Ok(tokens) => {
            for token in tokens {
                println!("{token}");
            }

            ExitValue::Success
        }
        scanner::Result::Err(error_sink, tokens) => {
            for err in error_sink.errors() {
                eprintln!("{err}");
            }
            for token in tokens {
                println!("{token}");
            }

            ExitValue::SyntaxError
        }
    }
}

fn parse(filename: &str) -> ExitValue {
    let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
        eprintln!("Failed to read file {}", filename);
        String::new()
    });

    let scanner = Scanner::new(&file_contents);
    let scanner::Result::Ok(tokens) = scanner.scan_tokens() else {
        return ExitValue::SyntaxError;
    };

    let mut parser = Parser::new(&tokens);
    let Ok(expr) = parser.parse() else {
        return ExitValue::SyntaxError;
    };

    let ast_printer = AstPrinter::new(&expr);
    println!("{}", ast_printer.print());

    ExitValue::Success
}
