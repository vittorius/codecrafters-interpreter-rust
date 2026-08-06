use crate::scanner::{self, Token};

pub trait Visitor<R> {
    fn visit_expr(&self, expr: &Expr) -> R;
}

pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        expr: Box<Expr>,
    },
    Literal {
        value: Option<scanner::Literal>,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
}

impl Expr {
    pub fn accept<R>(&self, visitor: &impl Visitor<R>) -> R {
        visitor.visit_expr(self)
    }

    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}
