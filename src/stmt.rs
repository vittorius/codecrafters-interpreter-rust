use std::{
    cell::{RefCell, RefMut},
    rc::Rc,
};

use crate::{environment::Environment, expr::Expr, scanner::Token};

pub trait VisitorMut<R> {
    fn visit_stmt(&mut self, stmt: &Stmt<'_>, env: Rc<RefCell<Environment<'_>>>) -> R;
}

// These variants own their Exprs because the latter ones
// are not being used anywhere besides being the part of their
// owning statements. Owned Exprs could not be references here
// because otherwise they would have to be references to
// temporary values that are dropped right after they are built.
#[derive(Debug)]
pub enum Stmt<'s> {
    Expression(Expr<'s>),
    Print(Expr<'s>),
    Var {
        token: Token<'s>,
        initializer: Option<Expr<'s>>,
    },
    Block(Vec<Stmt<'s>>),
}

impl<'s> Stmt<'s> {
    pub fn accept<R>(
        &self,
        visitor: &mut impl VisitorMut<R>,
        env: Rc<RefCell<Environment<'_>>>,
    ) -> R {
        visitor.visit_stmt(self, env)
    }

    // pub fn boxed(self) -> Box<Self> {
    //     Box::new(self)
    // }
}
