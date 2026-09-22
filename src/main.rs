use std::env;
use std::fs;
use std::io;
use std::io::Write;
use std::io::stdout;
use std::process::ExitCode;

use crossterm::ExecutableCommand;
use crossterm::cursor;
use crossterm::event;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyModifiers;
use crossterm::terminal;

use crate::ast_printer::AstPrinter;
use crate::console::RawModeGuard;
use crate::interpreter::Interpreter;
use crate::parser::Parser;
use crate::resolver::Resolver;
use crate::scanner::ScanError;
use crate::scanner::Scanner;

mod ast_printer;
mod callable;
mod console;
mod environment;
mod error;
mod expr;
mod function;
mod interpreter;
mod lox;
mod native;
mod parser;
mod resolver;
mod rpn_ast_printer;
mod scanner;
mod stmt;
mod token;
mod value;

#[repr(u8)]
enum ExitValue {
    Success = 0,
    Usage = 64,
    SyntaxError = 65, // lexical or syntactical grammar error
    RuntimeError = 70,
    Termination = 130,
}

impl From<io::Error> for ExitValue {
    fn from(_value: io::Error) -> Self {
        ExitValue::RuntimeError
    }
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
        match repl() {
            Ok(_) => ExitValue::Success,
            Err(err) => err,
        }
        .into()
    } else if args.len() == 3 {
        let command = &args[1];
        let filename = &args[2];

        let source = fs::read_to_string(filename).unwrap_or_else(|_| {
            eprintln!("Failed to read file {}", filename);
            String::new()
        });

        // TODO: reduce or eliminate code duplication between different command implementations
        match command.as_str() {
            "tokenize" => tokenize(&source).into(),
            "parse" => parse(&source).into(),
            "evaluate" => evaluate(&source).into(),
            "run" => run(&source).into(),
            _ => {
                eprintln!("Unknown command: {}", command);
                usage(&args[0]).into()
            }
        }
    } else {
        usage(&args[0]).into()
    }
}

fn usage(bin_name: &str) -> ExitValue {
    eprintln!(
        r#"
Usage: {0} (tokenize | parse | evaluate | run) <filename>
       {0} repl
"#,
        bin_name
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

    let mut ast_printer = AstPrinter::new(&expr);
    println!("{}", ast_printer.print());

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
    println!("{}", result);

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
    if let Err(err) = resolver.resolve_statements(&mut statements) {
        eprintln!("{err}");
        return ExitValue::RuntimeError;
    }

    match interpreter.interpret(&statements) {
        Ok(_) => ExitValue::Success,
        Err(err) => {
            eprintln!("{err}");
            ExitValue::RuntimeError
        }
    }
}

fn run_with_interpreter(source: &str, interpreter: &mut Interpreter) -> Result<(), String> {
    let scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens()?;

    let mut parser = Parser::new(tokens);
    let mut statements = parser.parse()?;

    let mut resolver = Resolver::new();
    resolver.resolve_statements(&mut statements)?;

    interpreter.interpret(&statements)?;

    Ok(())
}

// TODO: add syntax highlighting (or, at least, the prompt highlighting)
fn repl() -> Result<(), ExitValue> {
    fn move_cursor_to_prompt() -> io::Result<()> {
        stdout()
            .execute(cursor::MoveToColumn(PROMPT.len() as u16))
            .map(|_| ())
    }

    fn clear_to_prompt() -> io::Result<()> {
        move_cursor_to_prompt()?;
        stdout().execute(terminal::Clear(terminal::ClearType::UntilNewLine))?;

        Ok(())
    }

    let mut interpreter = Interpreter::new();
    let mut source = String::new();

    const PROMPT: &str = "> ";
    let mut history = Vec::<String>::new();
    let mut history_pos: usize = 0;

    let _raw_mode_guard = RawModeGuard::new()?;

    loop {
        print!("{}", PROMPT);
        stdout().flush()?;

        loop {
            if let Event::Key(key_event) = event::read()? {
                match (key_event.code, key_event.modifiers) {
                    // FIXME: preserve the currently-edited line in the history to enabling getting
                    // back to it with the down arrow but avoid duplicate entries of it in history
                    // after the subsequent up arrow.
                    (KeyCode::Up, _) => {
                        if history_pos == 0 {
                            continue;
                        }

                        clear_to_prompt()?;
                        history_pos -= 1;
                        print!("{}", history[history_pos]);
                        stdout().flush()?;
                        source = history[history_pos].clone();
                    }
                    (KeyCode::Down, _) => {
                        if history_pos == history.len().saturating_sub(1) {
                            continue;
                        }

                        clear_to_prompt()?;
                        history_pos += 1;
                        print!("{}", history[history_pos]);
                        stdout().flush()?;
                        source = history[history_pos].clone();
                    }
                    (KeyCode::Backspace, _) => {
                        if cursor::position()?.0 as usize > PROMPT.len() {
                            source.pop();
                            stdout()
                                .execute(cursor::MoveLeft(1))?
                                .execute(terminal::Clear(terminal::ClearType::UntilNewLine))?;
                        }
                    }
                    (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                        clear_to_prompt()?;
                        source.clear();
                    }
                    (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                        drop(_raw_mode_guard);
                        eprintln!("\nInterrupted, exiting");
                        return Err(ExitValue::Termination);
                    }
                    (KeyCode::Char(c), _) => {
                        source.push(c);
                        print!("{c}");
                        stdout().flush()?;
                    }
                    (KeyCode::Enter, _) => {
                        history.push(source.clone());
                        history_pos = history.len();

                        move_cursor_to_prompt()?;
                        println!();
                        break;
                    }
                    _ => {}
                }
            }
        }

        match run_with_interpreter(&source, &mut interpreter) {
            Ok(_) => {
                stdout().execute(cursor::MoveToColumn(0))?;
            }
            Err(msg) => {
                stdout().execute(cursor::MoveToColumn(0))?;
                for str in msg.split('\n') {
                    eprintln!("{str}");
                    stdout().execute(cursor::MoveToColumn(0))?;
                }
            }
        }
        source.clear();
    }
}
