use std::fmt::Display;

use crate::scanner::Literal::{Ident, Num, Str};

#[allow(non_camel_case_types)]
#[derive(Debug)]
enum TokenType {
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

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone)]
enum Literal {
    Ident(String),
    Str(String),
    Num(f64),
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ident(value) | Str(value) => write!(f, "{value}"),
            Num(value) => write!(f, "{value}"),
        }
    }
}

pub struct Token {
    token_type: TokenType,
    lexeme: String,
    literal: Option<Literal>,
    line: usize,
}

impl Token {
    fn new(token_type: TokenType, lexeme: String, literal: Option<Literal>, line: usize) -> Self {
        Self {
            token_type,
            lexeme,
            literal,
            line,
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: deal with clones and allocations here
        let literal = self
            .literal
            .clone()
            .map(|v| v.to_string())
            .unwrap_or_else(|| "null".to_owned());
        write!(f, "{} {} {}", self.token_type, self.lexeme, literal)
    }
}

pub struct Scanner {
    source: String, // TODO: try making this a &str
    tokens: Vec<Token>,
    start: usize,   // in bytes
    current: usize, // in bytes
    line: usize,
    pub has_error: bool,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Self {
            source,
            tokens: vec![],
            start: 0,
            current: 0,
            line: 1,
            has_error: false,
        }
    }

    pub fn scan_tokens(&mut self) {
        while !self.is_at_end() {
            self.start = self.current;

            self.scan_token();
        }

        self.tokens
            .push(Token::new(TokenType::EOF, "".to_owned(), None, self.line));
    }

    pub fn tokens(&self) -> &Vec<Token> {
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
            _ => {
                eprintln!("[line {}] Error: Unexpected character: {c}", self.line);
                self.has_error = true;
            }
        }
    }

    fn advance(&mut self) -> char {
        let c = self.source[self.current..]
            .chars()
            .next()
            .expect("Character cursor must not be at the source end");
        self.current += c.len_utf8();
        c
    }

    fn add_token(&mut self, token_type: TokenType) {
        self.add_token_with_literal(token_type, None);
    }

    fn add_token_with_literal(&mut self, token_type: TokenType, literal: Option<Literal>) {
        self.tokens.push(Token::new(
            token_type,
            self.source[self.start..self.current].to_owned(),
            literal,
            self.line,
        ));
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
}
