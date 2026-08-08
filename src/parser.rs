use crate::{
    expr::Expr,
    lox,
    scanner::{Literal, Token, TokenType, TokenType as TT},
};

// We follow the jlox code structure, therefore we report errors through a "global" function
// and don't return them as Err-s. Not very Rust-idiomatic but it's what it is.
pub type PResult<'a> = Result<Expr<'a>, ()>;

pub struct Parser<'a> {
    tokens: &'a [Token<'a>],
    current: usize,
    pub has_error: bool,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token<'a>]) -> Self {
        Parser {
            tokens,
            current: 0,
            has_error: false,
        }
    }

    pub fn parse(&mut self) -> Expr<'a> {
        match self.expression() {
            Ok(expr) => expr,
            Err(_) => {
                self.has_error = true;
                Expr::Literal(Literal::Nil)
            }
        }
    }

    fn peek(&self) -> &'a Token<'a> {
        &self.tokens[self.current]
    }

    // TODO: try to turn this token into a token eater/emitter
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

    fn consume(&mut self, token_type: &TokenType, message: &str) -> Result<&'a Token<'a>, ()> {
        if self.check(token_type) {
            return Ok(self.advance());
        };

        Self::error(self.peek(), message);
        Err(())
    }

    fn error(token: &Token, message: &str) {
        if token.token_type == TT::EOF {
            lox::error_at(token.line, " at end", message);
        } else {
            lox::error_at(token.line, &format!(" at '{}'", token.lexeme), message);
        }
    }

    fn expression(&mut self) -> PResult<'a> {
        self.equality()
    }

    fn equality(&mut self) -> PResult<'a> {
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

    fn comparison(&mut self) -> PResult<'a> {
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

    fn term(&mut self) -> PResult<'a> {
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

    fn factor(&mut self) -> PResult<'a> {
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

    fn unary(&mut self) -> PResult<'a> {
        if self.match_next(&[TT::BANG, TT::MINUS]) {
            let operator = *self.previous();
            let right = self.unary()?.boxed();
            return Ok(Expr::Unary { operator, right });
        }

        self.primary()
    }

    fn primary(&mut self) -> PResult<'a> {
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

        unreachable!("No 'primary' rule tokens matched.")
    }
}
