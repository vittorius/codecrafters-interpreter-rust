use crate::{environment::Env, expr::Expr, stmt::fun_decl::FunDecl, token::Token};

pub mod fun_decl;

pub trait VisitorEnv<R> {
    fn visit_stmt(&self, stmt: &Stmt, env: Env) -> R;
}

pub trait VisitorMut<R> {
    fn visit_stmt(&mut self, stmt: &Stmt) -> R;
}

// These variants own their Exprs because the latter ones
// are not being used anywhere besides being the part of their
// owning statements. Owned Exprs could not be references here
// because otherwise they would have to be references to
// temporary values that are dropped right after they are built.
#[derive(Debug, Clone)]
pub enum Stmt {
    Expression(Expr),
    Function(FunDecl),
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    Print(Expr),
    Return {
        keyword: Token,
        value: Expr,
    },
    Var {
        name: Token,
        initializer: Option<Expr>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    Block(Vec<Stmt>),
}

impl Stmt {
    pub fn accept_visitor_env<R>(&self, visitor: &impl VisitorEnv<R>, env: Env) -> R {
        visitor.visit_stmt(self, env)
    }

    pub fn accept_visitor_mut<R>(&self, visitor: &mut impl VisitorMut<R>) -> R {
        visitor.visit_stmt(self)
    }

    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}
