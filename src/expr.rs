#[cfg(feature = "lambda")]
use crate::expr::fun_expr::FunExpr;
use crate::{
    environment::Env,
    token::{self, Token},
};

pub mod fun_expr;

// pub use fun_expr::*;

pub trait Visitor<R> {
    fn visit_expr(&self, expr: &Expr, env: Env) -> R;
}

// Box<Expr> is used here instead of &Expr because the expression tree
// is built: the actual data must be allocated and owned by someone.
// If it's not a tree of boxed Exprs than it should've been a Vec or arena
// of Expr and the expression tree will be populated with references to it.
// It's deemed an overkill for our use-case, so we're going away with Box.
#[derive(Debug, Clone)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        paren: Token,
        arguments: Vec<Expr>,
    },
    #[cfg(feature = "conditional-op")]
    Conditional {
        cond: Box<Expr>,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Grouping(Box<Expr>),
    Literal(token::Literal),
    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Variable(Token), // token is the variable's name
    Assign {
        name: Token,
        value: Box<Expr>,
    },
    #[cfg(feature = "lambda")]
    Lambda(FunExpr),
}

impl Expr {
    pub fn accept<R>(&self, visitor: &impl Visitor<R>, env: Env) -> R {
        visitor.visit_expr(self, env)
    }

    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}
