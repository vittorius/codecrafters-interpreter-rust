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

// This type owns a string in `Str` variant because it's better for an interpreter
// where the environment must exist past the interpreted source line in REPL.
// A an AST type that borrows the source string is better for the compiler or
// for a one-shot interpreter (read the file, interpret, shutdown).
// Also, the reason for this is the ability to reuse the `Stmt` struct to store the
// functions' code in the persistent environment and execute it within the interpreter later.
// Otherwise, we would have to invent a parallel owned `Stmt` kind for that etc. etc.
// #[derive(Debug, Copy, Clone)]
#[derive(Debug, Clone)]
pub enum Literal {
    Str(String),
    Num(f64),
    Bool(bool),
    Nil,
}

impl Display for Literal {
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

// This type owns a string in `lexeme` because it's better for an interpreter
// where the environment must exist past the interpreted source line in REPL.
// A an AST type that borrows the source string is better for the compiler or
// for a one-shot interpreter (read the file, interpret, shutdown).
// Also, the reason for this is the ability to reuse the `Stmt` struct to store the
// functions' code in the persistent environment and execute it within the interpreter later.
// Otherwise, we would have to invent a parallel owned `Stmt` kind for that etc. etc.
#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Option<Literal>, // TODO: try to encode in types that Literal is present for token_type = NUMBER | STRING
    pub line: usize,
}

impl Token {
    pub fn new(
        token_type: TokenType,
        lexeme: String,
        literal: Option<Literal>,
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

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.token_type,
            self.lexeme,
            self.literal
                .as_ref()
                .map(|v| v.to_string())
                .unwrap_or_else(|| "null".to_owned())
        )
    }
}
