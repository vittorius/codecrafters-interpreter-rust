#![allow(dead_code, unused_variables)]

use std::env;
use std::fs;
use std::process::ExitCode;

use crate::ast_printer::AstPrinter;
use crate::parser::Parser;
use crate::scanner::Scanner;

mod ast_printer;
mod rpn_ast_printer;
mod expr;
mod lox;
mod parser;
mod scanner;

#[repr(u8)]
enum ExitValue {
    Success = 0,
    Usage = 64,
    SyntaxError = 65, // lexical or syntactical grammar error
    ParseError = 70,
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

    let mut scanner = Scanner::new(&file_contents);
    scanner.scan_tokens();

    for token in scanner.tokens() {
        println!("{token}");
    }

    if scanner.has_error {
        ExitValue::SyntaxError
    } else {
        ExitValue::Success
    }
}

fn parse(filename: &str) -> ExitValue {
    let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
        eprintln!("Failed to read file {}", filename);
        String::new()
    });

    let mut scanner = Scanner::new(&file_contents);
    scanner.scan_tokens();
    if scanner.has_error {
        return ExitValue::SyntaxError;
    }

    let mut parser = Parser::new(scanner.tokens());
    let expr = parser.parse();
    if parser.has_error {
        return ExitValue::ParseError;
    }

    let ast_printer = AstPrinter::new(&expr);
    println!("{}", ast_printer.print());

    ExitValue::Success
}
