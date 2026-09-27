use crate::{interpreter::environment::Env, stmt::Stmt};

pub trait VisitorEnv<R> {
    fn visit_stmt(&self, stmt: &Stmt, env: &Env) -> R;
}

pub trait VisitorMut<'a, R> {
    fn visit_stmt(&mut self, stmt: &'a mut Stmt) -> R;
}

impl Stmt {
    pub fn accept_visitor_env<R>(&self, visitor: &impl VisitorEnv<R>, env: &Env) -> R {
        visitor.visit_stmt(self, env)
    }

    pub fn accept_visitor_mut<'a, R>(&'a mut self, visitor: &mut impl VisitorMut<'a, R>) -> R {
        visitor.visit_stmt(self)
    }
}
