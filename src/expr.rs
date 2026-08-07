use crate::scanner::{self, Token};

pub trait Visitor<R> {
    fn visit_expr(&self, expr: &Expr) -> R;
}

// pub trait VisitorRef<R> {
//     fn visit_expr(&self, expr: &ExprRef) -> R;
// }

pub enum Expr<'a> {
    Binary {
        left: Box<Expr<'a>>,
        operator: Token<'a>,
        right: Box<Expr<'a>>,
    },
    Grouping {
        expr: Box<Expr<'a>>,
    },
    Literal {
        value: Option<scanner::Literal<'a>>,
    },
    Unary {
        operator: Token<'a>,
        right: Box<Expr<'a>>,
    },
}

impl<'a> Expr<'a> {
    pub fn accept<R>(&self, visitor: &impl Visitor<R>) -> R {
        visitor.visit_expr(self)
    }

    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}

// pub enum ExprRef<'a> {
//     Binary {
//         left: &'a ExprRef<'a>,
//         operator: &'a Token,
//         right: &'a ExprRef<'a>,
//     },
//     Grouping {
//         expr: &'a ExprRef<'a>,
//     },
//     Literal {
//         value: Option<&'a scanner::Literal>,
//     },
//     Unary {
//         operator: &'a Token,
//         right: &'a ExprRef<'a>,
//     },
// }

// impl<'a> ExprRef<'a> {
//     pub fn accept<R>(&self, visitor: &impl VisitorRef<R>) -> R {
//         visitor.visit_expr(self)
//     }
// }