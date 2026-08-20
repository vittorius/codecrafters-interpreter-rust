use crate::scanner::{self, Token};

pub trait Visitor<'e, R> {
    fn visit_expr(&self, expr: &'e Expr) -> R;
}

#[derive(Debug)]
pub enum Expr<'a> {
    Binary {
        left: Box<Expr<'a>>,
        operator: Token<'a>,
        right: Box<Expr<'a>>,
    },
    Conditional {
        cond: Box<Expr<'a>>,
        left: Box<Expr<'a>>,
        right: Box<Expr<'a>>,
    },
    Grouping(Box<Expr<'a>>),
    Literal(scanner::Literal<'a>),
    Unary {
        operator: Token<'a>,
        right: Box<Expr<'a>>,
    },
}

impl<'a> Expr<'a> {
    pub fn accept<R>(&'a self, visitor: &impl Visitor<'a, R>) -> R {
        visitor.visit_expr(self)
    }

    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}
