use std::{fmt::Display, marker::PhantomData};

use crate::{
    expr::{Expr, Visitor},
    scanner::{self, Token, TokenType as TT},
};

pub struct RuntimeError<'a> {
    token: Token<'a>,
    message: String,
}

pub type Result<'a> = std::result::Result<String, RuntimeError<'a>>;
type EvalResult<'a> = std::result::Result<Value, RuntimeError<'a>>;

// TODO: there could be an enum Object { Value, Ref }
// and Ref can hold stings and class objects, others go into Value
enum Value {
    Str(String),
    Num(f64),
    Bool(bool),
    Nil,
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Str(value) => write!(f, "{value}"),
            Value::Num(value) => write!(f, "{value}"),
            Value::Bool(value) => write!(f, "{value}"),
            Value::Nil => write!(f, "nil"),
        }
    }
}

pub struct Interpreter<'a> {
    _phantom: PhantomData<&'a ()>,
}

impl<'a> Interpreter<'a> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }

    pub fn interpret(&self, expr: &'a Expr<'a>) -> Result<'a> {
        self.evaluate(expr).map(|v| v.to_string())
    }

    fn evaluate(&self, expr: &'a Expr<'a>) -> EvalResult<'a> {
        expr.accept(self)
    }

    fn is_truthy(val: &Value) -> bool {
        match val {
            Value::Bool(b) => *b,
            Value::Nil => false,
            _ => true,
        }
    }

    fn is_equal(left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Nil, Value::Nil) => true,
            (Value::Nil, _) => false,
            (Value::Bool(l), Value::Bool(r)) => l == r,
            (Value::Num(l), Value::Num(r)) => l == r,
            (Value::Str(l), Value::Str(r)) => l == r,
            _ => false,
        }
    }

    fn visit_literal(literal: &scanner::Literal<'a>) -> EvalResult<'a> {
        Ok(match literal {
            scanner::Literal::Str(s) => Value::Str((*s).to_owned()),
            scanner::Literal::Num(n) => Value::Num(*n),
            scanner::Literal::Bool(b) => Value::Bool(*b),
            scanner::Literal::Nil => Value::Nil,
        })
    }

    fn visit_unary(&self, operator: &Token<'a>, expr: &'a Expr<'a>) -> EvalResult<'a> {
        let right = self.evaluate(expr)?;

        match (operator.token_type, right) {
            (TT::MINUS, Value::Num(n)) => Ok(Value::Num(-n)),
            (TT::BANG, val) => Ok(Value::Bool(!Self::is_truthy(&val))),
            _ => unreachable!(),
        }
    }

    fn visit_binary(
        &self,
        left: &'a Expr<'a>,
        operator: &'a Token<'a>,
        right: &'a Expr<'a>,
    ) -> EvalResult<'a> {
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;

        match (operator.token_type, left, right) {
            (TT::MINUS, Value::Num(l), Value::Num(r)) => Ok(Value::Num(l - r)),
            (TT::SLASH, Value::Num(l), Value::Num(r)) => Ok(Value::Num(l / r)),
            (TT::STAR, Value::Num(l), Value::Num(r)) => Ok(Value::Num(l * r)),
            (TT::PLUS, Value::Num(l), Value::Num(r)) => Ok(Value::Num(l + r)),
            (TT::PLUS, Value::Str(l), Value::Str(r)) => Ok(Value::Str(format!("{l}{r}"))),
            (TT::GREATER, Value::Num(l), Value::Num(r)) => Ok(Value::Bool(l > r)),
            (TT::GREATER_EQUAL, Value::Num(l), Value::Num(r)) => Ok(Value::Bool(l >= r)),
            (TT::LESS, Value::Num(l), Value::Num(r)) => Ok(Value::Bool(l < r)),
            (TT::LESS_EQUAL, Value::Num(l), Value::Num(r)) => Ok(Value::Bool(l <= r)),
            (TT::EQUAL_EQUAL, l, r) => Ok(Value::Bool(Self::is_equal(&l, &r))),
            (TT::BANG_EQUAL, l, r) => Ok(Value::Bool(!Self::is_equal(&l, &r))),
            _ => unreachable!(),
        }
    }
}

impl<'a> Visitor<'a, EvalResult<'a>> for Interpreter<'a> {
    fn visit_expr(&self, expr: &'a Expr) -> EvalResult<'a> {
        match expr {
            Expr::Binary {
                left,
                operator,
                right,
            } => self.visit_binary(left, operator, right),
            Expr::Conditional { cond, left, right } => todo!(),
            Expr::Grouping(expr) => self.evaluate(expr),
            Expr::Literal(literal) => Self::visit_literal(literal),
            Expr::Unary { operator, right } => self.visit_unary(operator, right),
        }
    }
}
