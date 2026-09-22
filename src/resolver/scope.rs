use std::collections::HashMap;

use crate::token::Token;

#[derive(Debug)]
pub struct VarData<'a> {
    status: VarStatus,
    #[cfg_attr(not(feature = "err-unused-vars"), allow(dead_code))]
    token: &'a Token,
}

#[derive(Debug, PartialEq, Eq)]
pub enum VarStatus {
    Declared,
    Defined,
    #[cfg(feature = "err-unused-vars")]
    Used,
}

pub struct Scope<'a> {
    vars: HashMap<&'a str, VarData<'a>>,
}

impl<'a> Scope<'a> {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }

    pub fn declare(&mut self, name: &'a Token) {
        self.vars.insert(
            &name.lexeme,
            VarData {
                status: VarStatus::Declared,
                token: name,
            },
        );
    }

    pub fn is_declared(&self, name: &Token) -> bool {
        self.vars.contains_key(name.lexeme.as_str())
    }

    pub fn define(&mut self, name: &'a Token) {
        self.vars
            .entry(&name.lexeme)
            .and_modify(|var_data| var_data.status = VarStatus::Defined);
    }

    pub fn is_defined(&self, name: &Token) -> bool {
        self.vars
            .get(name.lexeme.as_str())
            .is_some_and(|v| v.status == VarStatus::Defined)
    }

    #[cfg(feature = "err-unused-vars")]
    pub fn mark_used(&mut self, name: &Token) {
        if let Some(var_data) = self.vars.get_mut(name.lexeme.as_str()) {
            var_data.status = VarStatus::Used;
        }
    }

    #[cfg(feature = "err-unused-vars")]
    pub fn first_unused(&self) -> Option<&Token> {
        self.vars
            .iter()
            .find(|(_, var_data)| var_data.status != VarStatus::Used)
            .map(|(_, var_data)| var_data.token)
    }
}
