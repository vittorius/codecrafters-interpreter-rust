use crate::expr::Expr;

pub struct Parser<'a> {
    source: &'a str,
    pub has_error: bool,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        Parser {
            source,
            has_error: false,
        }
    }

    pub fn parse(&self) -> Expr {
        todo!()
    }
}
