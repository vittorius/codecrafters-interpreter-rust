#[cfg(feature = "lambdas")]
use crate::expr::fun_expr::FunExpr;
use crate::{
    environment::EnvShared,
    token::{self, Token},
};

pub mod fun_expr;

// TODO: experiment with turning this into a consuming visitor
pub trait VisitorEnv<R> {
    fn visit_expr(&self, expr: &Expr, env: EnvShared) -> R;
}

pub trait VisitorMut<'a, R> {
    fn visit_expr(&mut self, expr: &'a mut Expr) -> R;
}

// Box<Expr> is used here instead of &Expr because the expression tree
// is built: the actual data must be allocated and owned by someone.
// If it's not a tree of boxed Exprs than it should've been a Vec or arena
// of Expr and the expression tree will be populated with references to it.
// It's deemed an overkill for our use-case, so we're going away with Box.
//
// As for `depth` in Variable and Assign variants: since we clone our function declarations
// because they must outlive their source code for the sake of REPL, we cannot rely on Expr
// identity or structural equality (like in the book where Expr is a hash key). We do what
// the book already mentions: store the resolution information in the parse tree directly.
#[derive(Debug, Clone)]
pub enum Expr {
    Assign {
        name: Token,
        value: Box<Expr>,
        depth: Option<usize>, // delayed initialization by resolver; None is kept for globals
    },
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
    Get {
        object: Box<Expr>,
        name: Token,
    },
    #[cfg(feature = "conditional-op")]
    Conditional {
        cond: Box<Expr>,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Grouping(Box<Expr>),
    #[cfg(feature = "lambdas")]
    Lambda(FunExpr),
    Literal(token::Literal),
    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Set {
        object: Box<Expr>,
        name: Token,
        value: Box<Expr>,
    },
    This {
        keyword: Token,       // "this" keyword
        depth: Option<usize>, // same as for local variables
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Variable {
        name: Token,
        depth: Option<usize>, // delayed initialization by resolver; None is kept for globals
    },
}

impl Expr {
    pub fn accept_visitor_env<R>(&self, visitor: &impl VisitorEnv<R>, env: EnvShared) -> R {
        visitor.visit_expr(self, env)
    }

    pub fn accept_visitor_mut<'a, R>(&'a mut self, visitor: &mut impl VisitorMut<'a, R>) -> R {
        visitor.visit_expr(self)
    }

    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}
