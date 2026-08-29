use crate::{
    environment::Env,
    scanner::{self, Token},
};

pub trait VisitorMut<'a, R> {
    fn visit_expr(&mut self, expr: &'a Expr<'a>, env: Env<'a>) -> R;
}

// Box<Expr> is used here instead of &Expr because the expression tree
// is built: the actual data must be allocated and owned by someone.
// If it's not a tree of boxed Exprs than it should've been a Vec or arena
// of Expr and the expression tree will be populated with references to it.
// It's deemed an overkill for our use-case, so we're going away with Box.
#[derive(Debug)]
pub enum Expr<'e> {
    Binary {
        left: Box<Expr<'e>>,
        operator: Token<'e>,
        right: Box<Expr<'e>>,
    },
    Conditional {
        cond: Box<Expr<'e>>,
        left: Box<Expr<'e>>,
        right: Box<Expr<'e>>,
    },
    Grouping(Box<Expr<'e>>),
    Literal(scanner::Literal<'e>),
    Unary {
        operator: Token<'e>,
        right: Box<Expr<'e>>,
    },
    Variable(Token<'e>), // token is the variable's name
    Assign {
        name: Token<'e>,
        value: Box<Expr<'e>>,
    },
}

impl<'e> Expr<'e> {
    pub fn accept<R>(&'e self, visitor: &mut impl VisitorMut<'e, R>, env: Env<'e>) -> R {
        visitor.visit_expr(self, env)
    }

    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}
