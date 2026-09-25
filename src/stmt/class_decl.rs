use crate::{stmt::fun_decl::FunDecl, token::Token};

#[derive(Debug, Clone)]
pub struct ClassDecl {
    pub name: Token,
    pub methods: Vec<FunDecl>,
}
