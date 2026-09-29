use std::borrow::Cow;
use std::path::Path;

use rustyline::Cmd;
use rustyline::Editor;
use rustyline::Helper;
use rustyline::KeyCode;
use rustyline::KeyEvent;
use rustyline::Modifiers;
use rustyline::completion::Completer;
use rustyline::error::ReadlineError;
use rustyline::highlight::{CmdKind, Highlighter};
use rustyline::hint::Hinter;
use rustyline::history::DefaultHistory;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};

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
            // line comment: gray up to (excluding) the newline, so that the
            // following lines of a multiline buffer keep getting highlighted
            if let Some(end) = comment.find('\n') {
                highlighted.push_str(&colorize(&comment[..end], GRAY));
                rest = &comment[end..];
            } else {
                highlighted.push_str(&colorize(comment, GRAY));
                return highlighted;
            }
        } else if let Some(end) = comment.find("*/") {
            // block comment: gray up to the closing "*/"
            highlighted.push_str(&colorize(&comment[..end + 2], GRAY));
            rest = &comment[end + 2..];
        } else {
            // unterminated block comment: gray up to the end of the buffer
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

/// Lexical fragment check for multiline REPL input (idea borrowed from
/// evcxr's `scan.rs`): tells whether `source` is obviously cut off midway —
/// unclosed brackets, an unterminated string or block comment, or a trailing
/// operator / comma / dot / `else`. Such input keeps collecting lines on Enter
/// instead of being submitted.
///
/// Mirrors the `Scanner` semantics: block comments nest and may span multiple
/// lines; strings have no backslash escapes and may span multiple lines.
/// Deliberately conservative: if unsure, treat the input as complete so it gets
/// submitted and reported by the parser rather than trapping the user in a
/// continuation line.
fn is_incomplete_fragment(source: &str) -> bool {
    // Characters a valid Lox fragment can never end with (an open bracket is
    // covered by the bracket stack below)
    const TRAILING_OPERATOR: &[char] = &[
        '+', '-', '*', '/', '%', '=', '<', '>', '!', '&', '|', ',', '.',
    ];

    let mut brackets: Vec<char> = Vec::new();
    let mut chars = source.chars().peekable();
    // trailing context outside strings and comments, for the heuristics below
    let mut last_char: Option<char> = None;
    let mut last_word = String::new();
    let mut word_stale = false; // a whitespace gap precedes the next word char

    while let Some(c) = chars.next() {
        match c {
            '(' | '{' | '[' => {
                brackets.push(c);
                last_char = Some(c);
                last_word.clear();
            }
            ')' | '}' | ']' => {
                let opener = match c {
                    ')' => '(',
                    '}' => '{',
                    _ => '[',
                };
                if brackets.pop() != Some(opener) {
                    // Mismatched brackets: no additional input will fix this,
                    // so submit and let the parser report the error.
                    return false;
                }
                last_char = Some(c);
                last_word.clear();
            }
            '"' => {
                last_word.clear();
                // String: up to the next quote (the scanner has no escape
                // handling), and an unclosed one may still be completed
                if !chars.by_ref().any(|c| c == '"') {
                    return true;
                }
                last_char = Some('"');
            }
            '/' => match chars.peek() {
                Some('/') => {
                    // line comment: transparent to the trailing context
                    while matches!(chars.peek(), Some(&c) if c != '\n') {
                        chars.next();
                    }
                }
                Some('*') => {
                    // nested block comment, also transparent to the trailing
                    // context
                    chars.next(); // consume "*"
                    let mut depth = 1usize;
                    while depth > 0 {
                        let Some(c) = chars.next() else {
                            return true; // unterminated block comment
                        };
                        match (c, chars.peek()) {
                            ('*', Some('/')) => {
                                chars.next();
                                depth -= 1;
                            }
                            ('/', Some('*')) => {
                                chars.next();
                                depth += 1;
                            }
                            _ => {}
                        }
                    }
                }
                _ => {
                    last_char = Some('/');
                    last_word.clear();
                }
            },
            c if c.is_whitespace() => word_stale = true,
            c if c.is_ascii_alphanumeric() || c == '_' => {
                if word_stale {
                    last_word.clear();
                    word_stale = false;
                }
                last_word.push(c);
                last_char = Some(c);
            }
            _ => {
                last_char = Some(c);
                last_word.clear();
            }
        }
    }

    !brackets.is_empty()
        || last_word == "else"
        || matches!(last_char, Some(c) if TRAILING_OPERATOR.contains(&c))
}

/// Whether Enter should insert a newline instead of submitting `input`.
///
/// Escape hatch (evcxr-style): hammering Enter on an already empty
/// continuation line — at which point the buffer ends with "\n\n" —
/// force-submits whatever is there and lets the parser report the error.
fn should_continue(input: &str) -> bool {
    !input.ends_with("\n\n") && is_incomplete_fragment(input)
}

struct LoxHelper;

impl Completer for LoxHelper {
    type Candidate = String;
}

impl Hinter for LoxHelper {
    type Hint = String;
}

impl Validator for LoxHelper {
    fn validate(&self, ctx: &mut ValidationContext<'_>) -> Result<ValidationResult, ReadlineError> {
        if should_continue(ctx.input()) {
            Ok(ValidationResult::Incomplete)
        } else {
            Ok(ValidationResult::Valid(None))
        }
    }
}

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

    // Keys for a manual newline, regardless of how complete the input looks.
    // rustyline 18 doesn't parse the kitty keyboard protocol, so terminals
    // deliver "Ctrl+Enter" as either Ctrl+J (LF) or Alt+Enter (ESC CR); Ctrl+J
    // is also the classic Emacs newline binding. Enter+CTRL is there for the
    // Windows console.
    editor.bind_sequence(KeyEvent::from('\n'), Cmd::Newline);
    editor.bind_sequence(KeyEvent(KeyCode::Enter, Modifiers::ALT), Cmd::Newline);
    editor.bind_sequence(KeyEvent(KeyCode::Enter, Modifiers::CTRL), Cmd::Newline);

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
    use super::{
        AQUA, FG, GRAY, GREEN, PURPLE, RED, colorize, highlight_line, is_incomplete_fragment,
        should_continue,
    };

    /// Strips ANSI SGR escape sequences (`ESC[...m`) from `highlighted`.
    fn strip_ansi(highlighted: &str) -> String {
        let mut stripped = String::new();
        let mut chars = highlighted.chars();

        while let Some(c) = chars.next() {
            if c == '\x1b' {
                // skip until (and including) the terminating 'm' of the sequence
                for c in chars.by_ref() {
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

    #[test]
    fn line_comments_stop_at_the_newline_in_multiline_input() {
        let line = "print 1; // c\nprint 2;";
        let highlighted = highlight_line(line);

        assert!(highlighted.contains(&colorize("// c", GRAY)));
        // the code after the newline is still highlighted as code
        assert_eq!(highlighted.matches(&colorize("print", RED)).count(), 2);
        assert_eq!(strip_ansi(&highlighted), line);
    }

    #[test]
    fn block_comments_span_lines_in_multiline_input() {
        let line = "/* a\nb */ print 1;";
        let highlighted = highlight_line(line);

        assert!(highlighted.contains(&colorize("/* a\nb */", GRAY)));
        assert!(highlighted.contains(&colorize("print", RED)));
        assert_eq!(strip_ansi(&highlighted), line);
    }

    #[test]
    fn multiline_highlighting_preserves_the_visible_text() {
        let line = "fun f() {\n  return \"x\";\n}\n// tail";
        assert_eq!(strip_ansi(&highlight_line(line)), line);
    }

    #[test]
    fn complete_fragments_are_submitted() {
        for source in [
            "",
            "print 1;",
            "var x = 40 + 2;",
            "fun f() { return 1; }",
            "if (a) { b(); } else { c(); }",
            "class A < B { m() { return this; } }",
            r#"print "a } in a string";"#,
            "print 1; // a comment { with a bracket",
            "print 1; /* a comment } */",
            "print -5;",
            "print obj.field;",
            "print 4.5;",
            ":help",
            ":quit",
        ] {
            assert!(
                !is_incomplete_fragment(source),
                "should be complete: {source:?}"
            );
        }
    }

    #[test]
    fn incomplete_fragments_continue_the_line() {
        for source in [
            "fun f() {",
            "fun f(n) {",
            "if (a) {",
            "class A {",
            "print (1 + 2",
            "print 1 +",
            "var x =",
            "print obj.",
            "if (a) b; else",
            r#"print "unterminated"#,
            "/* unterminated block",
            "print 1; /* nested /* still open */",
            "fun f() { // a comment",
        ] {
            assert!(
                is_incomplete_fragment(source),
                "should be incomplete: {source:?}"
            );
        }
    }

    #[test]
    fn mismatched_brackets_are_submitted_for_a_parser_error() {
        for source in ["print 1);", "}", "fun f() }", "print (1];", r#""a" )"#] {
            assert!(
                !is_incomplete_fragment(source),
                "should not block submission: {source:?}"
            );
        }
    }

    #[test]
    fn double_enter_force_submits_an_incomplete_fragment() {
        // first Enter on the continuation line adds a newline...
        let continuation = "fun f() {\n";
        assert!(should_continue(continuation));
        // ...the second one submits the whole thing
        assert!(!should_continue("fun f() {\n\n"));
    }
}
