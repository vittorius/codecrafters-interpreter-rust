use crate::{environment::Env, expr::Expr, stmt::fun_decl::FunDecl, token::Token};

pub mod fun_decl;

// TODO: experiment with turning this into a consuming visitor
pub trait VisitorEnv<R> {
    fn visit_stmt(&self, stmt: &Stmt, env: Env) -> R;
}

pub trait VisitorMut<'a, R> {
    fn visit_stmt(&mut self, stmt: &'a mut Stmt) -> R;
}

// These variants own their Exprs because the latter ones
// are not being used anywhere besides being the part of their
// owning statements. Owned Exprs could not be references here
// because otherwise they would have to be references to
// temporary values that are dropped right after they are built.
#[derive(Debug, Clone)]
pub enum Stmt {
    Block(Vec<Stmt>),
    Class {
        name: Token,
        methods: Vec<FunDecl>,
    },
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
        // we could always return Value::Nil but None reflects the syntactical structure of 'return;' statement better
        value: Option<Expr>,
    },
    Var {
        name: Token,
        initializer: Option<Expr>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
}

impl Stmt {
    pub fn accept_visitor_env<R>(&self, visitor: &impl VisitorEnv<R>, env: Env) -> R {
        visitor.visit_stmt(self, env)
    }

    pub fn accept_visitor_mut<'a, R>(&'a mut self, visitor: &mut impl VisitorMut<'a, R>) -> R {
        visitor.visit_stmt(self)
    }

    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}
