use std::borrow::Cow;
use std::path::Path;

use rustyline::Editor;
use rustyline::Helper;
use rustyline::completion::Completer;
use rustyline::error::ReadlineError;
use rustyline::highlight::{CmdKind, Highlighter};
use rustyline::hint::Hinter;
use rustyline::history::DefaultHistory;
use rustyline::validate::Validator;

use crate::ExitValue;
use crate::interpreter::Interpreter;
use crate::interpreter::resolver::Resolver;
use crate::parser::Parser;
use crate::scanner::{self, Scanner};
use crate::token::TokenType;

const PROMPT: &str = "> ";
const HISTORY_FILE: &str = "history.txt";
const GOODBYE: &str = "Exiting. Bye-bye.";

const USAGE: &str = r"Hey, this is a Lox language REPL. Poke around and try some Lox scripts!

Commands:

- :help - shows this usage message
- :quit - quits the REPL";

// Colors of the Gruvbox Dark palette (https://github.com/morhetz/gruvbox).
struct Rgb(u8, u8, u8);

const GRAY: Rgb = Rgb(0x92, 0x83, 0x74); // comments
const RED: Rgb = Rgb(0xfb, 0x49, 0x34); // keywords
const GREEN: Rgb = Rgb(0xb8, 0xbb, 0x26); // strings
const PURPLE: Rgb = Rgb(0xd3, 0x86, 0x9b); // numbers and true/false/nil
const AQUA: Rgb = Rgb(0x8e, 0xc0, 0x7c); // operators and punctuation
const FG: Rgb = Rgb(0xeb, 0xdb, 0xb2); // identifiers
const ORANGE: Rgb = Rgb(0xfe, 0x80, 0x19); // the prompt

fn colorize(text: &str, Rgb(r, g, b): Rgb) -> String {
    // Truecolor ANSI SGR sequence; zero-width, so the display width of the
    // highlighted line stays the same as the original one (rustyline's
    // Highlighter contract).
    format!("\x1b[38;2;{r};{g};{b}m{text}\x1b[0m")
}

fn token_color(token_type: TokenType) -> Option<Rgb> {
    use TokenType as TT;

    Some(match token_type {
        TT::STRING => GREEN,
        TT::NUMBER | TT::TRUE | TT::FALSE | TT::NIL => PURPLE,
        TT::IDENTIFIER => FG,
        TT::AND
        | TT::CLASS
        | TT::ELSE
        | TT::FUN
        | TT::FOR
        | TT::IF
        | TT::OR
        | TT::PRINT
        | TT::RETURN
        | TT::SUPER
        | TT::THIS
        | TT::VAR
        | TT::WHILE => RED,
        TT::EOF => return None,
        // everything else is an operator or punctuation
        _ => AQUA,
    })
}

/// Byte index of the earliest comment start (`//` or `/*`) in `text`, if any.
fn next_comment_start(text: &str) -> Option<usize> {
    match (text.find("//"), text.find("/*")) {
        (Some(line), Some(block)) => Some(line.min(block)),
        (Some(idx), None) | (None, Some(idx)) => Some(idx),
        (None, None) => None,
    }
}

/// Colors comment spans within a between-tokens gap gray; everything else
/// (whitespace) is copied verbatim.
fn highlight_gap(gap: &str) -> String {
    let mut highlighted = String::new();
    let mut rest = gap;

    while let Some(idx) = next_comment_start(rest) {
        highlighted.push_str(&rest[..idx]);
        let comment = &rest[idx..];

        if comment.starts_with("//") {
            highlighted.push_str(&colorize(comment, GRAY));
            return highlighted;
        }

        // block comment: gray up to the closing "*/" or up to the line end
        if let Some(end) = comment.find("*/") {
            highlighted.push_str(&colorize(&comment[..end + 2], GRAY));
            rest = &comment[end + 2..];
        } else {
            highlighted.push_str(&colorize(comment, GRAY));
            return highlighted;
        }
    }

    highlighted.push_str(rest);
    highlighted
}

fn highlight_line(line: &str) -> String {
    // On a scan error (e.g. an unterminated string) the partial token list is
    // used to highlight the valid prefix; the malformed tail stays plain.
    let tokens = match Scanner::new(line).scan_tokens() {
        Ok(tokens) | Err(scanner::ScanError(_, tokens)) => tokens,
    };

    let mut highlighted = String::new();
    let mut pos = 0; // byte offset of the not-yet-emitted source prefix

    for token in &tokens {
        if token.token_type == TokenType::EOF {
            break;
        }

        highlighted.push_str(&highlight_gap(&line[pos..token.start]));

        let end = token.start + token.lexeme.len();
        let lexeme = &line[token.start..end];
        if let Some(color) = token_color(token.token_type) {
            highlighted.push_str(&colorize(lexeme, color));
        } else {
            highlighted.push_str(lexeme);
        }

        pos = end;
    }

    // the tail after the last token: a trailing comment or a malformed suffix
    highlighted.push_str(&highlight_gap(&line[pos..]));

    highlighted
}

struct LoxHelper;

impl Completer for LoxHelper {
    type Candidate = String;
}

impl Hinter for LoxHelper {
    type Hint = String;
}

impl Validator for LoxHelper {}

impl Helper for LoxHelper {}

impl Highlighter for LoxHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        Cow::Owned(highlight_line(line))
    }

    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        if default {
            Cow::Owned(colorize(prompt, ORANGE))
        } else {
            Cow::Borrowed(prompt)
        }
    }

    fn highlight_char(&self, _line: &str, _pos: usize, _kind: CmdKind) -> bool {
        true // re-highlight the line on every keystroke
    }
}

type LoxEditor = Editor<LoxHelper, DefaultHistory>;

pub fn run() -> ExitValue {
    let mut editor = match LoxEditor::new() {
        Ok(editor) => editor,
        Err(err) => {
            eprintln!("Failed to initialize the REPL: {err}");
            return ExitValue::RuntimeError;
        }
    };
    editor.set_helper(Some(LoxHelper));

    if Path::new(HISTORY_FILE).exists()
        && let Err(err) = editor.load_history(HISTORY_FILE)
    {
        eprintln!("Failed to load {HISTORY_FILE}: {err}");
    }

    let mut interpreter = Interpreter::new();

    loop {
        match editor.readline(PROMPT) {
            Ok(line) => {
                let input = line.trim();
                if input.is_empty() {
                    continue;
                }

                if let Err(err) = editor.add_history_entry(&line) {
                    eprintln!("Failed to add a history entry: {err}");
                }

                match input {
                    ":help" => println!("{USAGE}"),
                    ":quit" => return quit(&mut editor),
                    _ => {
                        if let Err(err) = run_with_interpreter(input, &mut interpreter) {
                            for err_line in err.lines() {
                                eprintln!("{err_line}");
                            }
                        }
                    }
                }
            }
            // Ctrl+C and Ctrl+D quit the REPL gracefully, just like :quit.
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => return quit(&mut editor),
            Err(err) => {
                eprintln!("REPL error: {err}");
                save_history(&mut editor);
                return ExitValue::RuntimeError;
            }
        }
    }
}

fn quit(editor: &mut LoxEditor) -> ExitValue {
    println!("{GOODBYE}");
    save_history(editor);
    ExitValue::Success
}

fn save_history(editor: &mut LoxEditor) {
    if let Err(err) = editor.save_history(HISTORY_FILE) {
        eprintln!("Failed to save {HISTORY_FILE}: {err}");
    }
}

fn run_with_interpreter(source: &str, interpreter: &mut Interpreter) -> Result<(), String> {
    let scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens()?;

    let mut parser = Parser::new(tokens);
    let mut statements = parser.parse()?;

    let mut resolver = Resolver::new();
    resolver.resolve(&mut statements)?;

    interpreter.interpret(&statements)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{AQUA, FG, GRAY, GREEN, PURPLE, RED, colorize, highlight_line};

    /// Strips ANSI SGR escape sequences (`ESC[...m`) from `highlighted`.
    fn strip_ansi(highlighted: &str) -> String {
        let mut stripped = String::new();
        let mut chars = highlighted.chars();

        while let Some(c) = chars.next() {
            if c == '\x1b' {
                // skip until (and including) the terminating 'm' of the sequence
                while let Some(c) = chars.next() {
                    if c == 'm' {
                        break;
                    }
                }
            } else {
                stripped.push(c);
            }
        }

        stripped
    }

    #[test]
    fn highlighting_preserves_the_visible_text() {
        let line = r#"var greeting = "hi" + 42; // a comment"#;
        assert_eq!(strip_ansi(&highlight_line(line)), line);
    }

    #[test]
    fn tokens_are_colored_by_category() {
        let highlighted = highlight_line(r#"var x = "s" + 42 + y;"#);

        assert!(highlighted.contains(&colorize("var", RED)));
        assert!(highlighted.contains(&colorize("x", FG)));
        assert!(highlighted.contains(&colorize("\"s\"", GREEN)));
        assert!(highlighted.contains(&colorize("42", PURPLE)));
        assert!(highlighted.contains(&colorize("+", AQUA)));
        assert!(highlighted.contains(&colorize("y", FG)));
    }

    #[test]
    fn comments_are_grayed() {
        let highlighted = highlight_line("print 1; /* block */ // line");

        assert!(highlighted.contains(&colorize("/* block */", GRAY)));
        assert!(highlighted.contains(&colorize("// line", GRAY)));
    }

    #[test]
    fn malformed_suffix_is_left_plain_but_prefix_is_highlighted() {
        // unterminated string: the valid prefix is still colored
        let line = r#"var x = 1; "oops"#;
        let highlighted = highlight_line(line);

        assert!(highlighted.contains(&colorize("var", RED)));
        assert_eq!(strip_ansi(&highlighted), line);
    }
}
