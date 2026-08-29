//! Lox grammar:
//!
//! program        → declaration* EOF ;
//! declaration    → varDecl | statement ;
//! varDecl        → "var" IDENTIFIER ( "=" expression )? ";" ;
//! statement      → exprStmt | printStmt | block ;
//! block          → "{" declaration* "}" ;
//! exprStmt       → expression ";"
//! expression     → assignment ;
//! assignment     → IDENTIFIER "=" assignment | compound ;
//! compound       → conditional ("," conditional)* ;
//! conditional    → equality ("?" equality ":" conditional)? ;
//! equality       → comparison ( ( "!=" | "==" ) comparison )* ;
//! comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
//! term           → factor ( ( "-" | "+" ) factor )* ;
//! factor         → unary ( ( "/" | "*" ) unary )* ;
//! unary          → ( "!" | "-" ) unary | primary ;
//! primary        → NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" | IDENTIFIER ;

use std::{error::Error, fmt::Display};

use crate::{
    expr::Expr,
    lox,
    scanner::{Literal, Token, TokenType, TokenType as TT},
    stmt::Stmt,
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
pub type Result<'a> = std::result::Result<Vec<Stmt<'a>>, ParseError>;
pub type ExprResult<'a> = std::result::Result<Expr<'a>, ParseError>;
type StmtResult<'a> = std::result::Result<Stmt<'a>, ParseError>;
type TokenResult<'a> = std::result::Result<&'a Token<'a>, ParseError>; // TODO: maybe it's more practical to pass token by value in this result type

pub struct Parser<'a> {
    tokens: &'a [Token<'a>],
    current: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token<'a>]) -> Self {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<'a> {
        let mut statements = Vec::<Stmt<'a>>::new();
        while !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        Ok(statements)
    }

    pub fn parse_expr(&mut self) -> ExprResult<'a> {
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

    // TODO: consider returning a Token copy
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

    // We need to construct the bare error and not wrap it into an Err variant
    // because this error value is used in different Result return types later
    fn mk_error(token: &Token<'_>, message: &str) -> ParseError {
        let msg = if token.token_type == TT::EOF {
            lox::fmt_error_at(token.line, " at end", message)
        } else {
            lox::fmt_error_at(token.line, &format!(" at '{}'", token.lexeme), message)
        };

        ParseError(msg)
    }

    fn declaration(&mut self) -> StmtResult<'a> {
        // TODO: later, any declaration must be sychronizable but not statements
        if self.match_next(&[TT::VAR]) {
            let decl = self.var_declaration();
            if decl.is_err() {
                self.synchronize();
            }
            decl
        } else {
            // we don't synchronize all statements
            // because we need to track an error condition
            // for "print" statement missing an expression, for example
            self.statement()
        }
    }

    fn var_declaration(&mut self) -> StmtResult<'a> {
        let name = self.consume(&TT::IDENTIFIER, "Expect variable name.")?;

        let mut initializer = None;
        if self.match_next(&[TT::EQUAL]) {
            initializer = Some(self.expression()?);
        }

        self.consume(&TT::SEMICOLON, "Expect ';' after variable declaration.")?;

        Ok(Stmt::Var {
            token: *name,
            initializer,
        })
    }

    fn statement(&mut self) -> StmtResult<'a> {
        if self.match_next(&[TT::PRINT]) {
            self.print_statement()
        } else if self.match_next(&[TT::LEFT_BRACE]) {
            self.block_statement()
        } else {
            self.expression_statement()
        }
    }

    fn print_statement(&mut self) -> StmtResult<'a> {
        let value = self.expression()?;
        self.consume(&TT::SEMICOLON, "Expect ';' after value.")?;
        Ok(Stmt::Print(value))
    }

    fn block_statement(&mut self) -> StmtResult<'a> {
        let mut statements: Vec<Stmt<'a>> = vec![];

        while !self.check(&TT::RIGHT_BRACE) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.consume(&TT::RIGHT_BRACE, "Expect '}' after block.")?;

        Ok(Stmt::Block(statements))
    }

    fn expression_statement(&mut self) -> StmtResult<'a> {
        let expr = self.expression()?;
        self.consume(&TT::SEMICOLON, "Expect ';' after expression.")?;
        Ok(Stmt::Expression(expr))
    }

    fn expression(&mut self) -> ExprResult<'a> {
        self.assignment()
    }

    fn assignment(&mut self) -> ExprResult<'a> {
        let expr = self.comma()?;

        if self.match_next(&[TT::EQUAL]) {
            let equals = *self.previous();
            let value = self.assignment()?;

            return if let Expr::Variable(name) = expr {
                Ok(Expr::Assign {
                    name,
                    value: value.boxed(),
                })
            } else {
                Err(Self::mk_error(&equals, "Invalid assignment target."))
            };
        }

        Ok(expr)
    }

    fn comma(&mut self) -> ExprResult<'a> {
        let mut expr = self.conditional()?;

        while self.match_next(&[TT::COMMA]) {
            let operator = *self.previous();
            let right = self.conditional()?.boxed();
            expr = Expr::Binary {
                left: expr.boxed(),
                operator,
                right,
            };
        }

        Ok(expr)
    }

    fn conditional(&mut self) -> ExprResult<'a> {
        let cond = self.equality()?;

        if self.match_next(&[TT::QUESTION]) {
            let left = self.equality()?.boxed();

            self.consume(
                &TT::COLON,
                "Expect ':' after then branch of conditional expression.",
            )?;
            let right = self.conditional()?.boxed();
            Ok(Expr::Conditional {
                cond: cond.boxed(),
                left,
                right,
            })
        } else {
            Ok(cond)
        }
    }

    fn equality(&mut self) -> ExprResult<'a> {
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

    fn comparison(&mut self) -> ExprResult<'a> {
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

    fn term(&mut self) -> ExprResult<'a> {
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

    fn factor(&mut self) -> ExprResult<'a> {
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

    fn unary(&mut self) -> ExprResult<'a> {
        if self.match_next(&[TT::BANG, TT::MINUS]) {
            let operator = *self.previous();
            let right = self.unary()?.boxed();
            return Ok(Expr::Unary { operator, right });
        }

        self.primary()
    }

    fn primary(&mut self) -> ExprResult<'a> {
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

        if self.match_next(&[TT::IDENTIFIER]) {
            return Ok(Expr::Variable(*self.previous()));
        }

        if self.match_next(&[TT::LEFT_PAREN]) {
            let expr = self.expression()?;
            self.consume(&TT::RIGHT_PAREN, "Expect ')' after expression.")?;
            return Ok(Expr::Grouping(expr.boxed()));
        }

        Err(Self::mk_error(self.peek(), "Expect expression."))
    }
}
