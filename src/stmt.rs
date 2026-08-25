use crate::expr::Expr;

pub trait Visitor<'s, R> {
    fn visit_stmt(&self, stmt: &'s Stmt) -> R;
}

pub enum Stmt<'a> {
    Expression(Expr<'a>),
    Print(Expr<'a>),
}

impl<'a> Stmt<'a> {
    pub fn accept<R>(&'a self, visitor: &impl Visitor<'a, R>) -> R {
        visitor.visit_stmt(self)
    }

    // pub fn boxed(self) -> Box<Self> {
    //     Box::new(self)
    // }
}
