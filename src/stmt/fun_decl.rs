use crate::{
    expr::fun_expr::{FunBody, FunExpr, FunParams},
    token::Token,
};

#[derive(Debug, Clone)]
pub struct FunDecl {
    pub name: Token,
    pub expr: FunExpr,
}

impl FunDecl {
    pub fn params(&self) -> &FunParams {
        &self.expr.params
    }
    
    pub fn body(&self) -> &FunBody {
        &self.expr.body
    }
}
