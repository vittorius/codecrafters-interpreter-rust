use crate::{stmt::Stmt, token::Token};

pub type FunParams = Vec<Token>;
pub type FunBody = Vec<Stmt>;

#[derive(Debug, Clone)]
pub struct FunExpr {
    pub params: FunParams,
    pub body: FunBody,
}
