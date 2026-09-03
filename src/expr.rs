use crate::{
    environment::Env,
    scanner::{self, Token},
};

pub trait Visitor<'a, R> {
    fn visit_expr(&self, expr: &'a Expr<'a>, env: Env) -> R;
}

// Box<Expr> is used here instead of &Expr because the expression tree
// is built: the actual data must be allocated and owned by someone.
// If it's not a tree of boxed Exprs than it should've been a Vec or arena
// of Expr and the expression tree will be populated with references to it.
// It's deemed an overkill for our use-case, so we're going away with Box.
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
    Logical {
        left: Box<Expr<'a>>,
        operator: Token<'a>,
        right: Box<Expr<'a>>,
    },
    Unary {
        operator: Token<'a>,
        right: Box<Expr<'a>>,
    },
    Variable(Token<'a>), // token is the variable's name
    Assign {
        name: Token<'a>,
        value: Box<Expr<'a>>,
    },
}

impl<'a> Expr<'a> {
    pub fn accept<R>(&'a self, visitor: &impl Visitor<'a, R>, env: Env) -> R {
        visitor.visit_expr(self, env)
    }

    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}
