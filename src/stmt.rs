use crate::{expr::Expr, scanner::Token};

pub trait VisitorMut<'s, R> {
    fn visit_stmt(&mut self, stmt: &'s Stmt) -> R;
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
}

impl<'a> Stmt<'a> {
    pub fn accept<R>(&'a self, visitor: &mut impl VisitorMut<'a, R>) -> R {
        visitor.visit_stmt(self)
    }

    // pub fn boxed(self) -> Box<Self> {
    //     Box::new(self)
    // }
}
