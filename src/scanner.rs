use std::{collections::HashMap, fmt::Display, iter::Peekable, str::Chars, sync::LazyLock};

use crate::lox;

#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TokenType {
    // Single-character tokens.
    LEFT_PAREN,
    RIGHT_PAREN,
    LEFT_BRACE,
    RIGHT_BRACE,
    COMMA,
    DOT,
    MINUS,
    PLUS,
    SEMICOLON,
    SLASH,
    STAR,

    // One or two character tokens.
    BANG,
    BANG_EQUAL,
    EQUAL,
    EQUAL_EQUAL,
    GREATER,
    GREATER_EQUAL,
    LESS,
    LESS_EQUAL,

    // Literals.
    IDENTIFIER,
    STRING,
    NUMBER,

    // Keywords.
    AND,
    CLASS,
    ELSE,
    FALSE,
    FUN,
    FOR,
    IF,
    NIL,
    OR,
    PRINT,
    RETURN,
    SUPER,
    THIS,
    TRUE,
    VAR,
    WHILE,

    EOF,
}

// TODO: refactor using phf crate
static KEYWORDS: LazyLock<HashMap<&str, TokenType>> = LazyLock::new(|| {
    use TokenType as TT;

    HashMap::from([
        ("and", TT::AND),
        ("class", TT::CLASS),
        ("else", TT::ELSE),
        ("false", TT::FALSE),
        ("for", TT::FOR),
        ("fun", TT::FUN),
        ("if", TT::IF),
        ("nil", TT::NIL),
        ("or", TT::OR),
        ("print", TT::PRINT),
        ("return", TT::RETURN),
        ("super", TT::SUPER),
        ("this", TT::THIS),
        ("true", TT::TRUE),
        ("var", TT::VAR),
        ("while", TT::WHILE),
    ])
});

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Literal<'a> {
    // Ident(&'a str),
    Str(&'a str),
    Num(f64),
    Bool(bool),
    Nil,
}

impl<'a> Display for Literal<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Literal::Ident(value) | Literal::Str(value) => write!(f, "{value}"),
            Literal::Str(value) => write!(f, "{value}"),
            Literal::Num(value) => {
                if value.fract() == 0.0 {
                    write!(f, "{value}.0")
                } else {
                    write!(f, "{value}")
                }
            },
            Literal::Bool(value) => write!(f, "{value}"),
            Literal::Nil => write!(f, "nil"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Token<'a> {
    pub token_type: TokenType,
    pub lexeme: &'a str,
    pub literal: Option<Literal<'a>>, // TODO: try to encode in types that Literal is present for token_type = NUMBER | STRING
    pub line: usize,
}

impl<'a> Token<'a> {
    pub fn new(
        token_type: TokenType,
        lexeme: &'a str,
        literal: Option<Literal<'a>>,
        line: usize,
    ) -> Self {
        Self {
            token_type,
            lexeme,
            literal,
            line,
        }
    }
}

impl<'a> Display for Token<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.token_type,
            self.lexeme,
            self.literal
                .map(|v| v.to_string())
                .unwrap_or_else(|| "null".to_owned())
        )
    }
}

pub struct Cursor<'a> {
    source: &'a str,
    chars: Peekable<Chars<'a>>,
    start_byte: usize,
    cur_byte: usize,
}

const EOF_CHAR: char = '\0';

impl<'a> Cursor<'a> {
    fn advance(&mut self) -> char {
        let c = self
            .chars
            .next()
            .expect("Character iterator must not be at the source end");
        self.cur_byte += c.len_utf8();
        c
    }

    fn catch_up(&mut self) {
        self.start_byte = self.cur_byte;
    }

    fn peek(&mut self) -> char {
        *self.chars.peek().unwrap_or(&EOF_CHAR)
    }

    fn peek_next(&mut self) -> char {
        let mut chars = self.chars.clone();
        chars.next();
        chars.next().unwrap_or(EOF_CHAR)
    }

    fn is_at_end(&mut self) -> bool {
        self.peek() == EOF_CHAR
    }
}

pub struct Scanner<'a> {
    cursor: Cursor<'a>,
    tokens: Vec<Token<'a>>,
    line: usize,
    pub has_error: bool, // TODO: remove this in favor of ErrorSink to collect all scanning errors
}

impl<'a> Scanner<'a> {
    // TODO: accept ErrorReporter to avoid calling lox::error and properly return Err up through the callstack
    pub fn new(source: &'a str) -> Self {
        Self {
            cursor: Cursor {
                source,
                chars: source.chars().peekable(),
                start_byte: 0,
                cur_byte: 0,
            },
            tokens: vec![],
            line: 1,
            has_error: false,
        }
    }

    pub fn scan_tokens(&mut self) {
        while !self.is_at_end() {
            self.cursor.catch_up();

            self.scan_token();
        }

        self.tokens
            .push(Token::new(TokenType::EOF, "", None, self.line));
    }

    pub fn tokens(&self) -> &Vec<Token<'a>> {
        &self.tokens
    }

    fn scan_token(&mut self) {
        use TokenType as TT;

        let c = self.advance();
        match c {
            '(' => self.add_token(TT::LEFT_PAREN),
            ')' => self.add_token(TT::RIGHT_PAREN),
            '{' => self.add_token(TT::LEFT_BRACE),
            '}' => self.add_token(TT::RIGHT_BRACE),
            ',' => self.add_token(TT::COMMA),
            '.' => self.add_token(TT::DOT),
            '-' => self.add_token(TT::MINUS),
            '+' => self.add_token(TT::PLUS),
            ';' => self.add_token(TT::SEMICOLON),
            '*' => self.add_token(TT::STAR),
            '!' => self.add_token_if('=', TT::BANG_EQUAL, TT::BANG),
            '=' => self.add_token_if('=', TT::EQUAL_EQUAL, TT::EQUAL),
            '<' => self.add_token_if('=', TT::LESS_EQUAL, TT::LESS),
            '>' => self.add_token_if('=', TT::GREATER_EQUAL, TT::GREATER),
            '/' => self.comment_or_slash(),
            ' ' | '\r' | '\t' => (),
            '\n' => self.line += 1,
            '"' => self.string(),
            _ => {
                if c.is_ascii_digit() {
                    self.number();
                } else if Self::is_alpha(&c) {
                    self.identifier();
                } else {
                    self.error(&format!("Unexpected character: {c}"));
                }
            }
        }
    }

    // IMPORTANT: accumulates the self.lexeme_cur
    // WARNING: will panic if cursor is at the end of the source
    fn advance(&mut self) -> char {
        self.cursor.advance()
    }

    // IMPORTANT: accumulates the self.lexeme_cur
    // WARNING: will panic if cursor is at the end of the source
    fn advance_twice(&mut self) -> (char, char) {
        let c = self.advance();
        (c, self.advance())
    }

    fn peek(&mut self) -> char {
        self.cursor.peek()
    }

    fn peek_next(&mut self) -> char {
        self.cursor.peek_next()
    }

    fn is_at_end(&mut self) -> bool {
        self.cursor.is_at_end()
    }

    fn cur_lexeme(&self) -> &'a str {
        &self.cursor.source[self.cursor.start_byte..self.cursor.cur_byte]
    }

    fn error(&mut self, message: &str) {
        lox::error(self.line, message);
        self.has_error = true;
    }

    fn add_token(&mut self, token_type: TokenType) {
        self.add_token_with_literal(token_type, None);
    }

    fn add_token_with_literal(&mut self, token_type: TokenType, literal: Option<Literal<'a>>) {
        self.tokens.push(Token::new(
            token_type,
            self.cur_lexeme(),
            literal,
            self.line,
        ));
    }

    /// IMPORTANT: accumulates the self.lexeme_cur
    fn add_token_if(&mut self, expected_next: char, left: TokenType, right: TokenType) {
        if self.cursor.peek() == expected_next {
            self.advance(); // consume next
            self.add_token(left);
        } else {
            self.add_token(right);
        }
    }

    fn comment_or_slash(&mut self) {
        let ch = self.peek();
        if ch == '/' {
            // single line comment
            while self.peek() != '\n' && !self.is_at_end() {
                self.advance();
            }
        } else if ch == '*' {
            // multiline comment
            self.advance(); // consume "*"

            let mut depth = 1;

            let mut c: char;
            loop {
                c = self.peek();
                let c_next = self.peek_next();

                match (c, c_next) {
                    ('\n', _) => {
                        self.line += 1;
                        self.advance();
                    }
                    ('*', '/') => {
                        depth -= 1;
                        self.advance_twice();
                    }
                    ('/', '*') => {
                        depth += 1;
                        self.advance_twice();
                    }
                    (EOF_CHAR, _) | (_, EOF_CHAR) => {
                        self.error("Unterminated multiline comment");
                        return;
                    }
                    _ => {
                        self.advance();
                    }
                }

                if depth == 0 {
                    break;
                }
            }
        } else {
            self.add_token(TokenType::SLASH);
        }
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            self.error("Unterminated string.");
            return;
        }

        self.advance(); // the closing "

        let string = self.cur_lexeme();
        self.add_token_with_literal(
            TokenType::STRING,
            Some(Literal::Str(&string[1..string.len() - 1])), // everything between quotes
        );
    }

    fn number(&mut self) {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            // found fractional part

            self.advance(); // consume "."

            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        self.add_token_with_literal(
            TokenType::NUMBER,
            Some(Literal::Num(self.cur_lexeme().parse().expect(
                "Number value validity must be guaranteed by tokenizing",
            ))),
        );
    }

    fn is_alpha(c: &char) -> bool {
        c.is_ascii_alphabetic() || *c == '_'
    }

    fn is_alphanumeric(c: &char) -> bool {
        Self::is_alpha(c) || c.is_ascii_digit()
    }

    fn identifier(&mut self) {
        while Self::is_alphanumeric(&self.peek()) {
            self.advance();
        }

        let token_type = if let Some(keyword_tt) = KEYWORDS.get(self.cur_lexeme()) {
            *keyword_tt
        } else {
            TokenType::IDENTIFIER
        };

        self.add_token(token_type);
    }
}
