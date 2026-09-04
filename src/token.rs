use std::{collections::HashMap, fmt::Display, sync::LazyLock};


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
    QUESTION,
    COLON,

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
    Str(&'a str),
    Num(f64),
    Bool(bool),
    Nil,
}

impl<'a> Display for Literal<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Str(value) => write!(f, "{value}"),
            Literal::Num(value) => {
                if value.fract() == 0.0 {
                    write!(f, "{value}.0")
                } else {
                    write!(f, "{value}")
                }
            }
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