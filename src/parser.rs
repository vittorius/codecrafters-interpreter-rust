//! Lox grammar:
//!
//! program        → declaration* EOF ;
//! declaration    → funDecl | varDecl | statement ;
//! funDecl        → "fun" function ;
//! function       → IDENTIFIER "(" parameters? ")" block ;
//! parameters     → IDENTIFIER ( "," IDENTIFIER )* ;/
//! varDecl        → "var" IDENTIFIER ( "=" expression )? ";" ;
//! statement      → exprStmt | forStmt | ifStmt | printStmt | whileStmt | block ;
//! exprStmt       → expression ";"
//! ifStmt         → "if" "(" expression ")" statement ( "else" statement )? ;
//! forStmt        → "for" "(" ( varDecl | exprStmt | ";" ) expression? ";" expression? ")" statement ;
//! printStmt      → "print" expression ";"
//! whileStmt      → "while" "(" expression ")" statement ;
//! block          → "{" declaration* "}" ;
//! expression     → comma ;
//! comma          → assignment ("," assignment)* ;
//! assignment     → IDENTIFIER "=" assignment | conditional ;
//! conditional    → logic_or ("?" logic_or ":" conditional)? ;
//! logic_or       → logic_and ( "or" logic_and )* ;
//! logic_and      → equality ( "and" equality )* ;
//! equality       → comparison ( ( "!=" | "==" ) comparison )* ;
//! comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
//! term           → factor ( ( "-" | "+" ) factor )* ;
//! factor         → unary ( ( "/" | "*" ) unary )* ;
//! unary          → ( "!" | "-" ) unary | call ;
//! call           → primary ( "(" arguments? ")" )* ;
//! arguments      → expression ( "," expression )* ;
//! primary        → NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" | IDENTIFIER ;

use std::{error::Error, fmt::Display};

use crate::{
    expr::Expr,
    lox,
    stmt::Stmt,
    token::{self, Literal, Token, TokenType, TokenType as TT},
};

#[derive(Debug)]
pub struct ParseError(pub String);

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
pub type Result = std::result::Result<Vec<Stmt>, ParseError>;
pub type ExprResult = std::result::Result<Expr, ParseError>;
type StmtResult = std::result::Result<Stmt, ParseError>;
type TokenResult = std::result::Result<Token, ParseError>;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

enum FunctionKind {
    Function,
    Method,
}

impl Display for FunctionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            FunctionKind::Function => write!(f, "function"),
            FunctionKind::Method => write!(f, "method"),
        }
    }
}

impl Parser {
    const FUN_ARGS_MAX: usize = 255;

    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result {
        let mut statements = Vec::<Stmt>::new();
        while !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        Ok(statements)
    }

    pub fn parse_expr(&mut self) -> ExprResult {
        self.expression()
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn match_next(&mut self, token_type: TokenType) -> bool {
        if self.check(token_type) {
            self.advance();
            return true;
        }

        false
    }

    fn match_next_any(&mut self, token_types: &[TokenType]) -> bool {
        token_types.iter().any(|tt| self.match_next(*tt))
    }

    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }

        self.peek().token_type == token_type
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }

        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TT::EOF
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> TokenResult {
        if self.check(token_type) {
            return Ok(self.advance().clone());
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
    fn mk_error(token: &Token, message: &str) -> ParseError {
        let msg = if token.token_type == TT::EOF {
            lox::fmt_error_at(token.line, " at end", message)
        } else {
            lox::fmt_error_at(token.line, &format!(" at '{}'", token.lexeme), message)
        };

        ParseError(msg)
    }

    fn declaration(&mut self) -> StmtResult {
        if self.match_next(TT::FUN) {
            self.function(FunctionKind::Function)
        } else if self.match_next(TT::VAR) {
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

    fn function(&mut self, kind: FunctionKind) -> StmtResult {
        let name = self
            .consume(TT::IDENTIFIER, &format!("Expect {} name.", kind))?
            .clone();
        self.consume(TT::LEFT_PAREN, &format!("Expect '(' after {} name.", kind))?;

        let mut params = Vec::<Token>::new();
        if !self.check(TT::RIGHT_PAREN) {
            loop {
                if params.len() >= Self::FUN_ARGS_MAX {
                    return Err(Self::mk_error(
                        self.peek(),
                        "Can't have more than 255 parameters.",
                    ));
                }
                params.push(
                    self.consume(TT::IDENTIFIER, "Expect parameter name.")?
                        .clone(),
                );

                if !self.match_next(TT::COMMA) {
                    break;
                }
            }
        }
        self.consume(TT::RIGHT_PAREN, "Expect ')' after parameters.")?;

        self.consume(
            TT::LEFT_BRACE,
            &format!("Expect '{{' before {} body.", kind),
        )?;
        let Stmt::Block(body) = self.block()? else {
            unreachable!("block statement must return a collection of statements")
        };

        Ok(Stmt::Function { name, params, body })
    }

    fn var_declaration(&mut self) -> StmtResult {
        let name = self.consume(TT::IDENTIFIER, "Expect variable name.")?;

        let mut initializer = None;
        if self.match_next(TT::EQUAL) {
            initializer = Some(self.expression()?);
        }

        self.consume(TT::SEMICOLON, "Expect ';' after variable declaration.")?;

        Ok(Stmt::Var {
            token: name,
            initializer,
        })
    }

    fn statement(&mut self) -> StmtResult {
        if self.match_next(TT::FOR) {
            self.for_statement()
        } else if self.match_next(TT::IF) {
            self.if_statement()
        } else if self.match_next(TT::PRINT) {
            self.print_statement()
        } else if self.match_next(TT::WHILE) {
            self.while_statement()
        } else if self.match_next(TT::LEFT_BRACE) {
            self.block()
        } else {
            self.expression_statement()
        }
    }

    fn expression_statement(&mut self) -> StmtResult {
        let expr = self.expression()?;
        self.consume(TT::SEMICOLON, "Expect ';' after expression.")?;

        Ok(Stmt::Expression(expr))
    }

    fn for_statement(&mut self) -> StmtResult {
        self.consume(TT::LEFT_PAREN, "Expect '(' after 'for'.")?;

        let initializer = if self.match_next(TT::SEMICOLON) {
            None
        } else if self.match_next(TT::VAR) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let condition = if !self.check(TT::SEMICOLON) {
            self.expression()?
        } else {
            Expr::Literal(token::Literal::Bool(true))
        };
        self.consume(TT::SEMICOLON, "Expect ';' after loop condition.")?;

        let increment = if !self.check(TT::RIGHT_PAREN) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(TT::RIGHT_PAREN, "Expect ')' after for clauses.")?;

        let mut body = self.statement()?;

        if let Some(increment) = increment {
            body = Stmt::Block(vec![body, Stmt::Expression(increment)])
        }

        body = Stmt::While {
            condition,
            body: body.boxed(),
        };

        if let Some(initializer) = initializer {
            body = Stmt::Block(vec![initializer, body]);
        }

        Ok(body)
    }

    fn if_statement(&mut self) -> StmtResult {
        self.consume(TT::LEFT_PAREN, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(TT::RIGHT_PAREN, "Expect ')' after if condition.")?;

        let then_branch = self.statement()?.boxed();
        let mut else_branch = None;
        if self.match_next(TT::ELSE) {
            else_branch = Some(self.statement()?.boxed());
        }

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn print_statement(&mut self) -> StmtResult {
        let value = self.expression()?;
        self.consume(TT::SEMICOLON, "Expect ';' after value.")?;

        Ok(Stmt::Print(value))
    }

    fn while_statement(&mut self) -> StmtResult {
        self.consume(TT::LEFT_PAREN, "Expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(TT::RIGHT_PAREN, "Expect ')' after condition.")?;
        let body = self.statement()?.boxed();

        Ok(Stmt::While { condition, body })
    }

    fn block(&mut self) -> StmtResult {
        let mut statements: Vec<Stmt> = vec![];

        while !self.check(TT::RIGHT_BRACE) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.consume(TT::RIGHT_BRACE, "Expect '}' after block.")?;

        Ok(Stmt::Block(statements))
    }

    fn expression(&mut self) -> ExprResult {
        self.comma()
    }

    fn comma(&mut self) -> ExprResult {
        let mut expr = self.assignment()?;

        while self.match_next(TT::COMMA) {
            let operator = self.previous().clone();
            let right = self.assignment()?.boxed();

            expr = Expr::Binary {
                left: expr.boxed(),
                operator,
                right,
            };
        }

        Ok(expr)
    }

    fn assignment(&mut self) -> ExprResult {
        let expr = self.conditional()?;

        if self.match_next(TT::EQUAL) {
            return if let Expr::Variable(name) = expr {
                Ok(Expr::Assign {
                    name,
                    value: self.assignment()?.boxed(),
                })
            } else {
                let equals = self.previous();
                Err(Self::mk_error(equals, "Invalid assignment target."))
            };
        }

        Ok(expr)
    }

    fn conditional(&mut self) -> ExprResult {
        let cond = self.or()?;

        if self.match_next(TT::QUESTION) {
            let left = self.or()?.boxed();

            self.consume(
                TT::COLON,
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

    fn or(&mut self) -> ExprResult {
        let mut expr = self.and()?;

        while self.match_next(TT::OR) {
            let operator = self.previous().clone();
            let right = self.and()?.boxed();

            expr = Expr::Logical {
                left: expr.boxed(),
                operator,
                right,
            };
        }

        Ok(expr)
    }

    fn and(&mut self) -> ExprResult {
        let mut expr = self.equality()?;

        while self.match_next(TT::AND) {
            let operator = self.previous().clone();
            let right = self.equality()?.boxed();

            expr = Expr::Logical {
                left: expr.boxed(),
                operator,
                right,
            }
        }

        Ok(expr)
    }

    fn equality(&mut self) -> ExprResult {
        let mut expr = self.comparison()?;

        while self.match_next_any(&[TT::BANG_EQUAL, TT::EQUAL_EQUAL]) {
            let operator = self.previous().clone();
            let right = self.comparison()?.boxed();

            expr = Expr::Binary {
                left: expr.boxed(),
                operator,
                right,
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> ExprResult {
        let mut expr = self.term()?;

        while self.match_next_any(&[TT::GREATER, TT::GREATER_EQUAL, TT::LESS, TT::LESS_EQUAL]) {
            let operator = self.previous().clone();
            let right = self.term()?.boxed();

            expr = Expr::Binary {
                left: expr.boxed(),
                operator,
                right,
            }
        }

        Ok(expr)
    }

    fn term(&mut self) -> ExprResult {
        let mut expr = self.factor()?;

        while self.match_next_any(&[TT::MINUS, TT::PLUS]) {
            let operator = self.previous().clone();
            let right = self.factor()?.boxed();

            expr = Expr::Binary {
                left: expr.boxed(),
                operator,
                right,
            }
        }

        Ok(expr)
    }

    fn factor(&mut self) -> ExprResult {
        let mut expr = self.unary()?;

        while self.match_next_any(&[TT::SLASH, TT::STAR]) {
            let operator = self.previous().clone();
            let right = self.unary()?.boxed();

            expr = Expr::Binary {
                left: expr.boxed(),
                operator,
                right,
            }
        }

        Ok(expr)
    }

    fn unary(&mut self) -> ExprResult {
        if self.match_next_any(&[TT::BANG, TT::MINUS]) {
            let operator = self.previous().clone();
            let right = self.unary()?.boxed();
            return Ok(Expr::Unary { operator, right });
        }

        self.call()
    }

    fn call(&mut self) -> ExprResult {
        let mut expr = self.primary()?;

        loop {
            if self.match_next(TT::LEFT_PAREN) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> ExprResult {
        let mut arguments = Vec::<Expr>::new();

        if !self.check(TT::RIGHT_PAREN) {
            loop {
                if arguments.len() >= 255 {
                    return Err(Self::mk_error(
                        self.peek(),
                        "Can't have more than 255 arguments.",
                    ));
                }
                arguments.push(self.expression()?);

                if !self.match_next(TT::COMMA) {
                    break;
                }
            }
        }

        let paren = self.consume(TT::RIGHT_PAREN, "Expect ')' after arguments.")?;

        Ok(Expr::Call {
            callee: callee.boxed(),
            paren,
            arguments,
        })
    }

    fn primary(&mut self) -> ExprResult {
        if self.match_next(TT::FALSE) {
            return Ok(Expr::Literal(Literal::Bool(false)));
        };
        if self.match_next(TT::TRUE) {
            return Ok(Expr::Literal(Literal::Bool(true)));
        };
        if self.match_next(TT::NIL) {
            return Ok(Expr::Literal(Literal::Nil));
        };

        if self.match_next_any(&[TT::NUMBER, TT::STRING]) {
            return Ok(Expr::Literal(
                self.previous()
                    .literal
                    .as_ref()
                    .expect("NUMBER or STRING must have literal assigned")
                    .clone(),
            ));
        }

        if self.match_next(TT::IDENTIFIER) {
            return Ok(Expr::Variable(self.previous().clone()));
        }

        if self.match_next(TT::LEFT_PAREN) {
            let expr = self.expression()?;
            self.consume(TT::RIGHT_PAREN, "Expect ')' after expression.")?;
            return Ok(Expr::Grouping(expr.boxed()));
        }

        Err(Self::mk_error(self.peek(), "Expect expression."))
    }
}
