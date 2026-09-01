use std::fmt::Display;

use crate::{
    environment::{BareEnv, Env, clone_env},
    error::RuntimeError,
    expr::{self, Expr},
    scanner::{self, Token, TokenType as TT},
    stmt::{self, Stmt},
};

// TODO: worth extracting into a separate module or moving to `environment`
// TODO: there could be an enum Object { Value, Ref }
// and Ref can hold stings and class objects, others go into Value.
//
// We made Value cloneable because we need to be able to store values in the environment
// and refer to variable in expressions. We construct a new Value in 2 cases: evaluating expressions
// and defining variables. Therefore, we cannot maintain a single place where values are stored.
// Actually, it's mostly because of String values, and we could have a dedicated string interner
// to own strings. String values would be Value::Str(&'v str). But it seems to be an overkill, and we
// just clone Strings (and Values) when we evaluate variables and get their values from the environment.
// The environment owns Values.
#[derive(Clone, Debug)]
pub enum Value {
    Str(String),
    Num(f64),
    Bool(bool),
    Nil,
}

type Void = (); // right now, trying to follow the book, maybe remove it later
const VOID: Void = ();

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

pub type Result = std::result::Result<Void, RuntimeError>;
pub type EvalResult = std::result::Result<String, RuntimeError>;
type StmtResult = std::result::Result<Void, RuntimeError>;
type ExprResult = std::result::Result<Value, RuntimeError>;

pub struct Interpreter {
    env: Env,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            env: BareEnv::new().wrapped(),
        }
    }

    pub fn with_env(env: Env) -> Self {
        Self { env }
    }

    pub fn interpret(&mut self, statements: &[Stmt<'_>]) -> Result {
        for stmt in statements {
            self.execute(stmt, clone_env(&self.env))?;
        }

        Ok(VOID)
    }

    pub fn interpret_expr(&mut self, expr: &Expr<'_>) -> EvalResult {
        self.evaluate(expr, clone_env(&self.env))
            .map(|v| v.to_string())
    }

    fn execute(&mut self, stmt: &Stmt<'_>, env: Env) -> StmtResult {
        stmt.accept(self, env)
    }

    fn execute_block(&mut self, statements: &[Stmt<'_>], env: Env) -> StmtResult {
        for stmt in statements {
            self.execute(stmt, clone_env(&env))?;
        }

        Ok(VOID)
    }

    fn evaluate(&mut self, expr: &Expr<'_>, env: Env) -> ExprResult {
        expr.accept(self, env)
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

    fn error(token: &Token<'_>, message: &str) -> ExprResult {
        Err(RuntimeError::new(token, message))
    }

    fn visit_literal<'l>(literal: &scanner::Literal<'l>) -> ExprResult {
        Ok(match literal {
            scanner::Literal::Str(s) => Value::Str((*s).to_owned()),
            scanner::Literal::Num(n) => Value::Num(*n),
            scanner::Literal::Bool(b) => Value::Bool(*b),
            scanner::Literal::Nil => Value::Nil,
        })
    }

    fn visit_unary(&mut self, operator: &Token<'_>, expr: &Expr<'_>, env: Env) -> ExprResult {
        let right = self.evaluate(expr, env)?;

        match (operator.token_type, right) {
            (TT::MINUS, Value::Num(n)) => Ok(Value::Num(-n)),
            (TT::MINUS, _) => Self::error(operator, "Operand must be a number."),
            (TT::BANG, val) => Ok(Value::Bool(!Self::is_truthy(&val))),
            _ => unreachable!(),
        }
    }

    fn visit_binary(
        &mut self,
        left: &Expr<'_>,
        operator: &Token<'_>,
        right: &Expr<'_>,
        env: Env,
    ) -> ExprResult {
        let left = self.evaluate(left, clone_env(&env))?;
        let right = self.evaluate(right, clone_env(&env))?;

        match (operator.token_type, left, right) {
            (TT::MINUS, Value::Num(l), Value::Num(r)) => Ok(Value::Num(l - r)),
            (TT::MINUS, _, _) => Self::error(operator, "Operands must be numbers."),
            (TT::SLASH, Value::Num(l), Value::Num(r)) => Ok(Value::Num(l / r)),
            (TT::SLASH, _, _) => Self::error(operator, "Operands must be numbers."),
            (TT::STAR, Value::Num(l), Value::Num(r)) => Ok(Value::Num(l * r)),
            (TT::STAR, _, _) => Self::error(operator, "Operands must be numbers."),
            (TT::PLUS, Value::Num(l), Value::Num(r)) => Ok(Value::Num(l + r)),
            (TT::PLUS, Value::Str(l), Value::Str(r)) => Ok(Value::Str(format!("{l}{r}"))),
            #[cfg(feature = "str-num-concat")]
            (TT::PLUS, Value::Num(l), Value::Str(r)) => Ok(Value::Str(format!("{l}{r}"))),
            #[cfg(feature = "str-num-concat")]
            (TT::PLUS, Value::Str(l), Value::Num(r)) => Ok(Value::Str(format!("{l}{r}"))),
            (TT::PLUS, _, _) => Self::error(operator, "Operands must be numbers."),
            (TT::GREATER, Value::Num(l), Value::Num(r)) => Ok(Value::Bool(l > r)),
            (TT::GREATER_EQUAL, Value::Num(l), Value::Num(r)) => Ok(Value::Bool(l >= r)),
            (TT::LESS, Value::Num(l), Value::Num(r)) => Ok(Value::Bool(l < r)),
            (TT::LESS_EQUAL, Value::Num(l), Value::Num(r)) => Ok(Value::Bool(l <= r)),
            #[cfg(feature = "str-cmp")]
            (TT::GREATER, Value::Str(l), Value::Str(r)) => Ok(Value::Bool(l > r)),
            #[cfg(feature = "str-cmp")]
            (TT::GREATER_EQUAL, Value::Str(l), Value::Str(r)) => Ok(Value::Bool(l >= r)),
            #[cfg(feature = "str-cmp")]
            (TT::LESS, Value::Str(l), Value::Str(r)) => Ok(Value::Bool(l < r)),
            #[cfg(feature = "str-cmp")]
            (TT::LESS_EQUAL, Value::Str(l), Value::Str(r)) => Ok(Value::Bool(l <= r)),
            (TT::GREATER | TT::GREATER_EQUAL | TT::LESS | TT::LESS_EQUAL, _, _) => {
                Self::error(operator, "Operands must be numbers.")
            }
            (TT::EQUAL_EQUAL, l, r) => Ok(Value::Bool(Self::is_equal(&l, &r))),
            (TT::BANG_EQUAL, l, r) => Ok(Value::Bool(!Self::is_equal(&l, &r))),
            (TT::COMMA, _, r) => Ok(r), // discard left and return right
            _ => unreachable!(),
        }
    }

    fn visit_conditional(
        &mut self,
        cond: &Expr<'_>,
        left: &Expr<'_>,
        right: &Expr<'_>,
        env: Env,
    ) -> ExprResult {
        if Self::is_truthy(&self.evaluate(cond, clone_env(&env))?) {
            self.evaluate(left, env)
        } else {
            self.evaluate(right, env)
        }
    }

    fn visit_variable(&self, name: &Token<'_>, env: Env) -> ExprResult {
        match env.borrow().get(name) {
            Some(value) => match value {
                #[cfg(feature = "init-vars")]
                Value::Nil => Self::error(
                    name,
                    &format!("Uninitialized variable \"{}\".", name.lexeme),
                ),
                _ => Ok(value),
            },
            None => Self::error(name, &format!("Undefined variable \"{}\".", name.lexeme)),
        }
    }

    fn visit_assign(&mut self, name: &Token<'_>, value: Value, env: Env) -> ExprResult {
        env.borrow_mut().assign(name, value)
    }
}

impl<'a> expr::VisitorMut<'a, ExprResult> for Interpreter {
    fn visit_expr(&mut self, expr: &'a Expr<'a>, env: Env) -> ExprResult {
        match expr {
            Expr::Binary {
                left,
                operator,
                right,
            } => self.visit_binary(left, operator, right, env),
            Expr::Conditional { cond, left, right } => {
                self.visit_conditional(cond, left, right, env)
            }
            Expr::Grouping(expr) => self.evaluate(expr, env),
            Expr::Literal(literal) => Self::visit_literal(literal),
            Expr::Unary { operator, right } => self.visit_unary(operator, right, env),
            Expr::Variable(name) => self.visit_variable(name, env),
            Expr::Assign { name, value } => {
                let value = self.evaluate(value, clone_env(&env))?;
                self.visit_assign(name, value, env)
            }
        }
    }
}

impl stmt::VisitorMut<StmtResult> for Interpreter {
    fn visit_stmt(&mut self, stmt: &Stmt<'_>, env: Env) -> StmtResult {
        match stmt {
            Stmt::Expression(expr) => self.evaluate(expr, env).map(|_| VOID),
            Stmt::Print(expr) => {
                println!("{}", self.evaluate(expr, env).map(|v| v.to_string())?);
                Ok(VOID)
            }
            Stmt::Var { token, initializer } => {
                let value = match initializer {
                    Some(expr) => self.evaluate(expr, clone_env(&env))?,
                    None => Value::Nil,
                };

                env.borrow_mut().define(token, value);

                Ok(VOID)
            }
            Stmt::Block(statements) => {
                self.execute_block(
                    statements,
                    BareEnv::with_enclosing(clone_env(&env)).wrapped(),
                )?;

                Ok(VOID)
            }
        }
    }
}
