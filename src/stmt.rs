use crate::{environment::Env, expr::Expr, scanner::Token};

pub trait Visitor<R> {
    fn visit_stmt(&self, stmt: &Stmt<'_>, env: Env) -> R;
}

// These variants own their Exprs because the latter ones
// are not being used anywhere besides being the part of their
// owning statements. Owned Exprs could not be references here
// because otherwise they would have to be references to
// temporary values that are dropped right after they are built.
#[derive(Debug)]
pub enum Stmt<'a> {
    Expression(Expr<'a>),
    Function {
        name: Token<'a>,
        params: Vec<Token<'a>>,
        body: Vec<Stmt<'a>>,
    },
    If {
        condition: Expr<'a>,
        then_branch: Box<Stmt<'a>>,
        else_branch: Option<Box<Stmt<'a>>>,
    },
    Print(Expr<'a>),
    Var {
        token: Token<'a>,
        initializer: Option<Expr<'a>>,
    },
    While {
        condition: Expr<'a>,
        body: Box<Stmt<'a>>,
    },
    Block(Vec<Stmt<'a>>),
}

impl<'a> Stmt<'a> {
    pub fn accept<R>(&'a self, visitor: &impl Visitor<R>, env: Env) -> R {
        visitor.visit_stmt(self, env)
    }

    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}
