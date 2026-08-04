use std::{fmt::Display, iter::Peekable, str::Chars};

use crate::lox;

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
            Literal::Ident(value) | Literal::Str(value) => write!(f, "{value}"),
            Literal::Num(value) => write!(f, "{value}"),
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

pub struct Scanner<'a> {
    chars: Peekable<Chars<'a>>,
    tokens: Vec<Token>,
    lexeme_cur: String,
    line: usize,
    pub has_error: bool,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            chars: source.chars().peekable(),
            tokens: vec![],
            lexeme_cur: String::with_capacity(2),
            line: 1,
            has_error: false,
        }
    }

    pub fn scan_tokens(&mut self) {
        while !self.is_at_end() {
            self.lexeme_cur.clear();

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
            '!' => self.add_token_if('=', TT::BANG_EQUAL, TT::BANG),
            '=' => self.add_token_if('=', TT::EQUAL_EQUAL, TT::EQUAL),
            '<' => self.add_token_if('=', TT::LESS_EQUAL, TT::LESS),
            '>' => self.add_token_if('=', TT::GREATER_EQUAL, TT::GREATER),
            '/' => self.add_comment_or_slash(),
            ' ' | '\r' | '\t' => (),
            '\n' => self.line += 1,
            '"' => self.add_string(),
            _ => {
                self.error(&format!("Unexpected character: {c}"));
            }
        }
    }

    /// IMPORTANT: accumulates the self.lexeme_cur
    fn advance(&mut self) -> char {
        let c = self
            .chars
            .next()
            .expect("Character iterator must not be at the source end");
        self.lexeme_cur.push(c);
        c
    }

    fn peek(&mut self) -> Option<&char> {
        self.chars.peek()
    }

    fn add_token(&mut self, token_type: TokenType) {
        self.add_token_with_literal(token_type, None);
    }

    fn add_token_with_literal(&mut self, token_type: TokenType, literal: Option<Literal>) {
        self.tokens.push(Token::new(
            token_type,
            self.lexeme_cur.clone(),
            literal,
            self.line,
        ));
    }

    /// IMPORTANT: accumulates the self.lexeme_cur
    fn add_token_if(&mut self, expected: char, left: TokenType, right: TokenType) {
        if let Some(ch_next) = self.peek()
            && *ch_next == expected
        {
            let ch_clone = *ch_next;
            self.lexeme_cur.push(ch_clone); // the 1st char of the lexeme was pushed to the buffer in `advance`
            self.chars.next();
            self.add_token(left);
        } else {
            self.add_token(right);
        }
    }

    fn add_comment_or_slash(&mut self) {
        if let Some(ch_next) = self.peek()
            && *ch_next == '/'
        {
            while let Some(ch) = self.peek()
                && *ch != '\n'
            {
                self.advance();
            }
        } else {
            self.add_token(TokenType::SLASH);
        }
    }

    fn add_string(&mut self) {
        while let Some(ch) = self.peek()
            && *ch != '"'
        {
            if *ch == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.peek().is_none() {
            self.error("Unterminated string.");
            return;
        }

        self.advance(); // the closing "

        self.add_token_with_literal(
            TokenType::STRING,
            Some(Literal::Str(
                self.lexeme_cur[1..self.lexeme_cur.len() - 1].to_owned(),
            )),
        );
    }

    fn is_at_end(&mut self) -> bool {
        self.peek().is_none()
    }

    fn error(&mut self, message: &str) {
        lox::error(self.line, message);
        self.has_error = true;
    }
}
