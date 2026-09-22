use crate::{
    expr::fun_expr::FunExpr,
    token::Token,
};

#[derive(Debug, Clone)]
pub struct FunDecl {
    pub name: Token,
    pub expr: FunExpr,
}
