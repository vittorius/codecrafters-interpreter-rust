use crate::{
    expr::Expr,
    scanner::{Token, TokenType, TokenType as TT},
};

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
        todo!()
    }

    fn peek(&self) -> &'a Token<'a> {
        &self.tokens[self.current]
    }

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

    fn expression(&mut self) -> Expr<'a> {
        self.equality()
    }

    fn equality(&mut self) -> Expr<'a> {
        let mut expr = self.comparison();

        while self.match_next(&[TT::BANG_EQUAL, TT::EQUAL_EQUAL]) {
            let operator = *self.previous();
            let right = self.comparison();
            expr = Expr::Binary {
                left: expr.boxed(),
                operator,
                right: right.boxed(),
            };
        }

        expr
    }

    fn comparison(&mut self) -> Expr<'a> {
        todo!()
    }
}
