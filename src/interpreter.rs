use std::fmt::Display;

use crate::{
    expr::{Expr, Visitor},
    scanner::{self, Token, TokenType as TT},
};

pub struct RuntimeError<'a> {
    token: Token<'a>,
    message: String,
}

pub type Result<'a> = std::result::Result<String, RuntimeError<'a>>;
type EvalResult<'a> = std::result::Result<Value<'a>, RuntimeError<'a>>;

// TODO: there could be an enum Object { Value, Ref }
// and Ref can hold stings and class objects, others go into Value
enum Value<'a> {
    // Str(String),
    Str(&'a str), // let's try references first
    Num(f64),
    Bool(bool),
    Nil,
}

impl<'a> Display for Value<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Str(value) => write!(f, "{value}"),
            Value::Num(value) => write!(f, "{value}"),
            Value::Bool(value) => write!(f, "{value}"),
            Value::Nil => write!(f, "nil"),
        }
    }
}

pub struct Interpreter {}

impl Interpreter {
    pub fn new() -> Self {
        Self {}
    }

    pub fn interpret<'a>(&self, expr: &'a Expr<'a>) -> Result<'a> {
        self.evaluate(expr).map(|v| v.to_string())
    }

    fn evaluate<'a>(&self, expr: &'a Expr<'a>) -> EvalResult<'a> {
        expr.accept(self)
    }

    fn visit_literal<'a>(&self, literal: &scanner::Literal<'a>) -> Value<'a> {
        match literal {
            scanner::Literal::Str(s) => Value::Str(s),
            scanner::Literal::Num(n) => Value::Num(*n),
            scanner::Literal::Bool(b) => Value::Bool(*b),
            scanner::Literal::Nil => Value::Nil,
        }
    }

    fn visit_unary<'a>(&self, token: &Token<'a>, right: &'a Expr<'a>) -> EvalResult<'a> {
        match (token.token_type, right) {
            (TT::MINUS, Expr::Literal(scanner::Literal::Num(n))) => Ok(Value::Num(-n)),
            (TT::BANG, Expr::Literal(lit)) => Ok(Value::Bool(!Self::is_truthy(lit))),
            _ => unreachable!(),
        }
    }

    fn is_truthy(lit: &scanner::Literal) -> bool {
        match lit {
            scanner::Literal::Bool(b) => *b,
            scanner::Literal::Nil => false,
            _ => true,
        }
    }
}

impl<'a> Visitor<'a, EvalResult<'a>> for Interpreter {
    fn visit_expr(&self, expr: &'a Expr) -> EvalResult<'a> {
        match expr {
            Expr::Binary {
                left,
                operator,
                right,
            } => todo!(),
            Expr::Conditional { cond, left, right } => todo!(),
            Expr::Grouping(expr) => self.evaluate(expr),
            Expr::Literal(literal) => Ok(self.visit_literal(literal)),
            Expr::Unary { operator, right } => todo!(),
        }
    }
}
