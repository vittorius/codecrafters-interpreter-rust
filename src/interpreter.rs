use std::{fmt::Display, marker::PhantomData};

use crate::{
    expr::{Expr, Visitor},
    lox,
    scanner::{self, Token, TokenType as TT},
};

pub struct RuntimeError(String);

pub type Result = std::result::Result<String, RuntimeError>;
type EvalResult = std::result::Result<Value, RuntimeError>;

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

    pub fn interpret(&self, expr: &'a Expr<'a>) -> Result {
        self.evaluate(expr).map(|v| v.to_string())
    }

    fn evaluate(&self, expr: &'a Expr<'a>) -> EvalResult {
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

    fn error(token: &Token, message: &str) -> EvalResult {
        Err(RuntimeError(lox::fmt_runtime_error(token.line, message)))
    }

    fn visit_literal(literal: &scanner::Literal<'a>) -> EvalResult {
        Ok(match literal {
            scanner::Literal::Str(s) => Value::Str((*s).to_owned()),
            scanner::Literal::Num(n) => Value::Num(*n),
            scanner::Literal::Bool(b) => Value::Bool(*b),
            scanner::Literal::Nil => Value::Nil,
        })
    }

    fn visit_unary(&self, operator: &Token<'a>, expr: &'a Expr<'a>) -> EvalResult {
        let right = self.evaluate(expr)?;

        match (operator.token_type, right) {
            (TT::MINUS, Value::Num(n)) => Ok(Value::Num(-n)),
            (TT::MINUS, _) => Self::error(operator, "Operand must be a number."),
            (TT::BANG, val) => Ok(Value::Bool(!Self::is_truthy(&val))),
            _ => unreachable!(),
        }
    }

    fn visit_binary(
        &self,
        left: &'a Expr<'a>,
        operator: &'a Token<'a>,
        right: &'a Expr<'a>,
    ) -> EvalResult {
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;

        match (operator.token_type, left, right) {
            (TT::MINUS, Value::Num(l), Value::Num(r)) => Ok(Value::Num(l - r)),
            (TT::MINUS, _, _) => Self::error(operator, "Operands must be numbers."),
            (TT::SLASH, Value::Num(l), Value::Num(r)) => Ok(Value::Num(l / r)),
            (TT::SLASH, _, _) => Self::error(operator, "Operands must be numbers."),
            (TT::STAR, Value::Num(l), Value::Num(r)) => Ok(Value::Num(l * r)),
            (TT::STAR, _, _) => Self::error(operator, "Operands must be numbers."),
            (TT::PLUS, Value::Num(l), Value::Num(r)) => Ok(Value::Num(l + r)),
            (TT::PLUS, Value::Str(l), Value::Str(r)) => Ok(Value::Str(format!("{l}{r}"))),
            (TT::PLUS, _, _) => Self::error(operator, "Operands must be numbers."),
            (TT::GREATER, Value::Num(l), Value::Num(r)) => Ok(Value::Bool(l > r)),
            (TT::GREATER_EQUAL, Value::Num(l), Value::Num(r)) => Ok(Value::Bool(l >= r)),
            (TT::LESS, Value::Num(l), Value::Num(r)) => Ok(Value::Bool(l < r)),
            (TT::LESS_EQUAL, Value::Num(l), Value::Num(r)) => Ok(Value::Bool(l <= r)),
            (TT::GREATER | TT::GREATER_EQUAL | TT::LESS | TT::LESS_EQUAL, _, _) => {
                Self::error(operator, "Operands must be numbers.")
            }
            (TT::EQUAL_EQUAL, l, r) => Ok(Value::Bool(Self::is_equal(&l, &r))),
            (TT::BANG_EQUAL, l, r) => Ok(Value::Bool(!Self::is_equal(&l, &r))),
            _ => unreachable!(),
        }
    }
}

impl<'a> Visitor<'a, EvalResult> for Interpreter<'a> {
    fn visit_expr(&self, expr: &'a Expr) -> EvalResult {
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
