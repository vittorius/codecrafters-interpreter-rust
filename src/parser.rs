use crate::{
    expr::Expr,
    scanner::{Token, TokenType, TokenType as TT},
};

pub struct Parser<'a> {
    tokens: &'a [Token],
    current: usize,
    pub has_error: bool,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Parser {
            tokens,
            current: 0,
            has_error: false,
        }
    }

    pub fn parse(&self) -> Expr {
        todo!()
    }

    fn matches(token_types: &[TokenType]) -> bool {
        for tt in token_types {
            if check(tt) {
                advance();
                return true;
            }
        }

        false
    }

    fn previous(&self) -> Token {
        // it seems that cloning the token it inevitable
        // because of the definition of Expr and because of Token contains String's
        self.tokens[self.current - 1].clone()
    }

    fn expression(&self) -> Expr {
        equality(self)
    }

    fn equality(&self) -> Expr {
        let expr = comparison(self);

        while Self::matches(&[TT::BANG_EQUAL, TT::EQUAL_EQUAL]) {
            let operator = self.previous();
        }
    }

    fn comparison(&self) -> Expr {
        todo!()
    }
}
