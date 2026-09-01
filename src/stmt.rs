use crate::{environment::Env, expr::Expr, scanner::Token};

pub trait VisitorMut<R> {
    fn visit_stmt(&mut self, stmt: &Stmt<'_>, env: Env) -> R;
}

// These variants own their Exprs because the latter ones
// are not being used anywhere besides being the part of their
// owning statements. Owned Exprs could not be references here
// because otherwise they would have to be references to
// temporary values that are dropped right after they are built.
#[derive(Debug)]
pub enum Stmt<'a> {
    Expression(Expr<'a>),
    Print(Expr<'a>),
    Var {
        token: Token<'a>,
        initializer: Option<Expr<'a>>,
    },
    Block(Vec<Stmt<'a>>),
}

impl<'a> Stmt<'a> {
    pub fn accept<R>(&'a self, visitor: &mut impl VisitorMut<R>, env: Env) -> R {
        visitor.visit_stmt(self, env)
    }
}
