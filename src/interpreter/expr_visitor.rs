use crate::{expr::Expr, interpreter::environment::Env};

pub trait VisitorEnv<R> {
    fn visit_expr(&self, expr: &Expr, env: &Env) -> R;
}

pub trait VisitorMut<'a, R> {
    fn visit_expr(&mut self, expr: &'a mut Expr) -> R;
}

impl Expr {
    pub fn accept_visitor_env<R>(&self, visitor: &impl VisitorEnv<R>, env: &Env) -> R {
        visitor.visit_expr(self, env)
    }

    pub fn accept_visitor_mut<'a, R>(&'a mut self, visitor: &mut impl VisitorMut<'a, R>) -> R {
        visitor.visit_expr(self)
    }
}
