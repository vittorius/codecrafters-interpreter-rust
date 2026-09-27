use crate::expr::Expr;

pub trait Visitor<R> {
    fn visit_expr(&self, expr: &Expr) -> R;
}

impl Expr {
    pub fn accept_visitor<R>(&self, visitor: &impl Visitor<R>) -> R {
        visitor.visit_expr(self)
    }
}
