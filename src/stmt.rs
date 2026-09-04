use crate::{environment::Env, expr::Expr, token::Token};

pub trait Visitor<R> {
    fn visit_stmt(&self, stmt: &Stmt, env: Env) -> R;
}

#[derive(Debug, Clone)]
pub struct FunctionDeclaration {
    pub name: Token,
    pub params: Vec<Token>,
    pub body: Vec<Stmt>,
}

// These variants own their Exprs because the latter ones
// are not being used anywhere besides being the part of their
// owning statements. Owned Exprs could not be references here
// because otherwise they would have to be references to
// temporary values that are dropped right after they are built.
#[derive(Debug, Clone)]
pub enum Stmt {
    Expression(Expr),
    Function(FunctionDeclaration),
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    Print(Expr),
    Var {
        token: Token,
        initializer: Option<Expr>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    Block(Vec<Stmt>),
}

impl Stmt {
    pub fn accept<R>(&self, visitor: &impl Visitor<R>, env: Env) -> R {
        visitor.visit_stmt(self, env)
    }

    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}
