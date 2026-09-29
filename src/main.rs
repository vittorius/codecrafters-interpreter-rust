use std::env;
use std::fs;
use std::process::ExitCode;

use crate::interpreter::Interpreter;
use crate::interpreter::resolver::Resolver;
use crate::parser::Parser;
use crate::printer::ast_printer::AstPrinter;
use crate::scanner::ScanError;
use crate::scanner::Scanner;

mod error;
mod expr;
mod interpreter;
mod lox;
mod parser;
mod printer;
mod repl;
mod scanner;
mod stmt;
mod token;

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
    if args.len() == 2 && args[1] == "repl" {
        repl::run().into()
    } else if args.len() == 3 {
        let command = &args[1];
        let filename = &args[2];

        let source = fs::read_to_string(filename).unwrap_or_else(|_| {
            eprintln!("Failed to read file {filename}");
            String::new()
        });

        match command.as_str() {
            "tokenize" => tokenize(&source).into(),
            "parse" => parse(&source).into(),
            "evaluate" => evaluate(&source).into(),
            "run" => run(&source).into(),
            _ => {
                eprintln!("Unknown command: {command}");
                usage(&args[0]).into()
            }
        }
    } else {
        usage(&args[0]).into()
    }
}

fn usage(bin_name: &str) -> ExitValue {
    eprintln!(
        r"
Usage: {bin_name} (tokenize | parse | evaluate | run) <filename>
       {bin_name} repl
"
    );

    ExitValue::Usage
}

fn tokenize(source: &str) -> ExitValue {
    let scanner = Scanner::new(source);
    match scanner.scan_tokens() {
        Ok(tokens) => {
            for token in tokens {
                println!("{token}");
            }

            ExitValue::Success
        }
        Err(ScanError(error_sink, tokens)) => {
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

fn parse(source: &str) -> ExitValue {
    let scanner = Scanner::new(source);
    let scanner::Result::Ok(tokens) = scanner.scan_tokens() else {
        return ExitValue::SyntaxError;
    };

    let mut parser = Parser::new(tokens);
    let Ok(expr) = parser.parse_expr() else {
        return ExitValue::SyntaxError;
    };

    let ast_printer = AstPrinter;
    println!("{}", ast_printer.print(&expr));

    ExitValue::Success
}

fn evaluate(source: &str) -> ExitValue {
    let scanner = Scanner::new(source);
    let scanner::Result::Ok(tokens) = scanner.scan_tokens() else {
        return ExitValue::SyntaxError;
    };

    let mut parser = Parser::new(tokens);
    let Ok(expr) = parser.parse_expr() else {
        return ExitValue::SyntaxError;
    };

    let interpreter = Interpreter::new();
    let Ok(result) = interpreter.interpret_expr(&expr) else {
        return ExitValue::RuntimeError;
    };
    println!("{result}");

    ExitValue::Success
}

fn run(source: &str) -> ExitValue {
    let scanner = Scanner::new(source);
    let scanner::Result::Ok(tokens) = scanner.scan_tokens() else {
        return ExitValue::SyntaxError;
    };

    let mut parser = Parser::new(tokens);

    let mut statements = match parser.parse() {
        Ok(res) => res,
        Err(err) => {
            eprintln!("{err}");
            return ExitValue::SyntaxError;
        }
    };

    let interpreter = Interpreter::new();

    let mut resolver = Resolver::new();
    if let Err(err) = resolver.resolve(&mut statements) {
        eprintln!("{err}");
        return ExitValue::SyntaxError;
    }

    match interpreter.interpret(&statements) {
        Ok(()) => ExitValue::Success,
        Err(err) => {
            eprintln!("{err}");
            ExitValue::RuntimeError
        }
    }
}
