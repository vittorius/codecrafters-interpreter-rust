//! Lox grammar:
//!
//! expression     → compound ;
//! compound       → ternary ("," ternary)* ;
//! ternary        → equality ("?" equality ":" equality)* ;
//! equality       → comparison ( ( "!=" | "==" ) comparison )* ;
//! comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
//! term           → factor ( ( "-" | "+" ) factor )* ;
//! factor         → unary ( ( "/" | "*" ) unary )* ;
//! unary          → ( "!" | "-" ) unary | primary ;
//! primary        → NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" ;

use std::{error::Error, fmt::Display};

use crate::{
    expr::Expr,
    lox,
    scanner::{Literal, Token, TokenType, TokenType as TT},
};

#[derive(Debug)]
pub struct ParseError(String);

impl Error for ParseError {}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// We don't follow the jlox code structure here for the sake of writing idiomatic Rust.
// Instead of reporting error immediately (actually, just printing it),
// we accumulate them in the error sink. Also, instead of throwing an error,
// we use Result returns values and don't panic.
pub type Result<'a> = std::result::Result<Expr<'a>, ParseError>;

type TokenResult<'a> = std::result::Result<&'a Token<'a>, ParseError>;

pub struct Parser<'a> {
    tokens: &'a [Token<'a>],
    current: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token<'a>]) -> Self {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<'a> {
        self.expression()
    }

    fn peek(&self) -> &'a Token<'a> {
        &self.tokens[self.current]
    }

    // TODO: try to turn this into a token eater/emitter
    // and offload the matching to the Rust `match` in rule functions
    fn match_next(&mut self, token_types: &[TokenType]) -> bool {
        for tt in token_types {
            if self.check(tt) {
                self.advance();
                return true;
            }
        }

        false
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }

        self.peek().token_type == *token_type
    }

    fn advance(&mut self) -> &'a Token<'a> {
        if !self.is_at_end() {
            self.current += 1;
        }

        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TT::EOF
    }

    fn previous(&self) -> &'a Token<'a> {
        &self.tokens[self.current - 1]
    }

    fn consume(&mut self, token_type: &TokenType, message: &str) -> TokenResult<'a> {
        if self.check(token_type) {
            return Ok(self.advance());
        };

        Err(Self::mk_error(self.peek(), message))
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == TT::SEMICOLON {
                return;
            }

            match self.peek().token_type {
                TT::CLASS
                | TT::FUN
                | TT::VAR
                | TT::FOR
                | TT::IF
                | TT::WHILE
                | TT::PRINT
                | TT::RETURN => return,
                _ => {
                    self.advance();
                }
            }
        }
    }

    fn mk_error(token: &Token, message: &str) -> ParseError {
        let msg = if token.token_type == TT::EOF {
            lox::fmt_error_at(token.line, " at end", message)
        } else {
            lox::fmt_error_at(token.line, &format!(" at '{}'", token.lexeme), message)
        };

        ParseError(msg)
    }

    fn expression(&mut self) -> Result<'a> {
        let mut expr = self.ternary()?;

        while self.match_next(&[TT::COMMA]) {
            let operator = *self.previous();
            let right = self.ternary()?.boxed();
            expr = Expr::Binary {
                left: expr.boxed(),
                operator,
                right,
            };
        }

        Ok(expr)
    }

    fn ternary(&mut self) -> Result<'a> {
        let cond = self.equality()?;

        if self.match_next(&[TT::QUESTION]) {
            let left = self.equality()?.boxed();

            if self.match_next(&[TT::COLON]) {
                let right = self.ternary()?.boxed();

                Ok(Expr::Ternary {
                    cond: cond.boxed(),
                    left,
                    right,
                })
            } else {
                Err(Self::mk_error(
                    self.peek(),
                    "Unterminated ternary expression",
                ))
            }
        } else {
            Ok(cond)
        }
    }

    fn equality(&mut self) -> Result<'a> {
        let mut expr = self.comparison()?;

        while self.match_next(&[TT::BANG_EQUAL, TT::EQUAL_EQUAL]) {
            let operator = *self.previous();
            let right = self.comparison()?.boxed();
            expr = Expr::Binary {
                left: expr.boxed(),
                operator,
                right,
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<'a> {
        let mut expr = self.term()?;

        while self.match_next(&[TT::GREATER, TT::GREATER_EQUAL, TT::LESS, TT::LESS_EQUAL]) {
            let operator = *self.previous();
            let right = self.term()?.boxed();
            expr = Expr::Binary {
                left: expr.boxed(),
                operator,
                right,
            }
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<'a> {
        let mut expr = self.factor()?;

        while self.match_next(&[TT::MINUS, TT::PLUS]) {
            let operator = *self.previous();
            let right = self.factor()?.boxed();
            expr = Expr::Binary {
                left: expr.boxed(),
                operator,
                right,
            }
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<'a> {
        let mut expr = self.unary()?;

        while self.match_next(&[TT::SLASH, TT::STAR]) {
            let operator = *self.previous();
            let right = self.unary()?.boxed();
            expr = Expr::Binary {
                left: expr.boxed(),
                operator,
                right,
            }
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<'a> {
        if self.match_next(&[TT::BANG, TT::MINUS]) {
            let operator = *self.previous();
            let right = self.unary()?.boxed();
            return Ok(Expr::Unary { operator, right });
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<'a> {
        if self.match_next(&[TT::FALSE]) {
            return Ok(Expr::Literal(Literal::Bool(false)));
        };
        if self.match_next(&[TT::TRUE]) {
            return Ok(Expr::Literal(Literal::Bool(true)));
        };
        if self.match_next(&[TT::NIL]) {
            return Ok(Expr::Literal(Literal::Nil));
        };

        if self.match_next(&[TT::NUMBER, TT::STRING]) {
            return Ok(Expr::Literal(
                self.previous()
                    .literal
                    .expect("NUMBER or STRING must have literal assigned"),
            ));
        }

        if self.match_next(&[TT::LEFT_PAREN]) {
            let expr = self.expression()?;
            self.consume(&TT::RIGHT_PAREN, "Expect ')' after expression.")?;
            return Ok(Expr::Grouping(expr.boxed()));
        }

        Err(Self::mk_error(self.peek(), "Expect expression."))
    }
}
