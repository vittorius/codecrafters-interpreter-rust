use std::rc::Rc;

use crate::{
    environment::{BareEnv, Env, clone_env},
    error::RuntimeError,
    expr::{self, Expr},
    native::ClockFunction,
    scanner::{self, Token, TokenType as TT},
    stmt::{self,  Stmt},
    value::Value::{self, Callable},
};

type Void = (); // right now, trying to follow the book, maybe remove it later
const VOID: Void = ();

pub type Result = std::result::Result<Void, RuntimeError>;
pub type StringResult = std::result::Result<String, RuntimeError>;
type StmtResult = std::result::Result<Void, RuntimeError>;
type ExprResult = std::result::Result<Value, RuntimeError>;

pub struct Interpreter {
    env: Env,
    global_env: Env,
}

impl Interpreter {
    pub fn new() -> Self {
        let env = BareEnv::new().wrapped();
        env.borrow_mut()
            .define("clock", Value::Callable(Rc::new(ClockFunction)));
        Self {
            global_env: clone_env(&env),
            env,
        }
    }

    pub fn interpret(&self, statements: &[Stmt<'_>]) -> Result {
        for stmt in statements {
            self.execute(stmt, clone_env(&self.env))?;
        }

        Ok(VOID)
    }

    pub fn interpret_expr(&self, expr: &Expr<'_>) -> StringResult {
        self.evaluate(expr, clone_env(&self.env))
            .map(|v| v.to_string())
    }

    fn execute(&self, stmt: &Stmt<'_>, env: Env) -> StmtResult {
        stmt.accept(self, env)
    }

    fn execute_block(&self, statements: &[Stmt<'_>], env: Env) -> StmtResult {
        for stmt in statements {
            self.execute(stmt, clone_env(&env))?;
        }

        Ok(VOID)
    }

    fn evaluate(&self, expr: &Expr<'_>, env: Env) -> ExprResult {
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

    fn visit_logical(
        &self,
        left: &Expr<'_>,
        operator: &Token<'_>,
        right: &Expr<'_>,
        env: Env,
    ) -> ExprResult {
        let left = self.evaluate(left, clone_env(&env))?;

        if operator.token_type == TT::OR {
            if Self::is_truthy(&left) {
                return Ok(left);
            };
        } else {
            if !Self::is_truthy(&left) {
                return Ok(left);
            };
        }

        self.evaluate(right, env)
    }

    fn visit_unary(&self, operator: &Token<'_>, expr: &Expr<'_>, env: Env) -> ExprResult {
        let right = self.evaluate(expr, env)?;

        match (operator.token_type, right) {
            (TT::MINUS, Value::Num(n)) => Ok(Value::Num(-n)),
            (TT::MINUS, _) => Self::error(operator, "Operand must be a number."),
            (TT::BANG, val) => Ok(Value::Bool(!Self::is_truthy(&val))),
            _ => unreachable!(),
        }
    }

    fn visit_binary(
        &self,
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
            _ => unreachable!("Invalid binary operation."),
        }
    }

    fn visit_call(
        &self,
        callee: &Expr<'_>,
        paren: &Token<'_>,
        arguments: &[Expr<'_>],
        env: Env,
    ) -> ExprResult {
        let callee = self.evaluate(callee, clone_env(&env))?;

        let arguments = arguments
            .iter()
            .map(|arg| self.evaluate(arg, clone_env(&env)))
            .collect::<std::result::Result<Vec<_>, _>>()?;

        if let Callable(function) = callee {
            if arguments.len() != function.arity() {
                return Self::error(
                    paren,
                    &format!(
                        "Expected {} arguments but got {}.",
                        function.arity(),
                        arguments.len()
                    ),
                );
            }

            Ok(function.call(self, &arguments, env))
        } else {
            Self::error(paren, "Can only call functions and classes.")
        }
    }

    fn visit_conditional(
        &self,
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

    fn visit_assign(&self, name: &Token<'_>, value: Value, env: Env) -> ExprResult {
        env.borrow_mut().assign(name, value)
    }

    fn visit_if_statement(
        &self,
        condition: &Expr<'_>,
        then_branch: &Stmt<'_>,
        else_branch: &Option<Box<Stmt<'_>>>,
        env: Env,
    ) -> StmtResult {
        if Self::is_truthy(&self.evaluate(condition, clone_env(&env))?) {
            self.execute(then_branch, env)?;
        } else if let Some(else_branch) = else_branch {
            self.execute(else_branch, env)?;
        }
        Ok(VOID)
    }

    fn visit_while_statement(&self, condition: &Expr<'_>, body: &Stmt<'_>, env: Env) -> StmtResult {
        while Self::is_truthy(&self.evaluate(condition, clone_env(&env))?) {
            self.execute(body, clone_env(&env))?
        }

        Ok(VOID)
    }
}

impl<'a> expr::Visitor<'a, ExprResult> for Interpreter {
    fn visit_expr(&self, expr: &'a Expr<'a>, env: Env) -> ExprResult {
        match expr {
            Expr::Binary {
                left,
                operator,
                right,
            } => self.visit_binary(left, operator, right, env),
            Expr::Call {
                callee,
                paren,
                arguments,
            } => self.visit_call(callee, paren, arguments, env),
            Expr::Conditional { cond, left, right } => {
                self.visit_conditional(cond, left, right, env)
            }
            Expr::Grouping(expr) => self.evaluate(expr, env),
            Expr::Literal(literal) => Self::visit_literal(literal),
            Expr::Logical {
                left,
                operator,
                right,
            } => self.visit_logical(left, operator, right, env),
            Expr::Unary { operator, right } => self.visit_unary(operator, right, env),
            Expr::Variable(name) => self.visit_variable(name, env),
            Expr::Assign { name, value } => {
                let value = self.evaluate(value, clone_env(&env))?;
                self.visit_assign(name, value, env)
            }
        }
    }
}

impl stmt::Visitor<StmtResult> for Interpreter {
    fn visit_stmt(&self, stmt: &Stmt<'_>, env: Env) -> StmtResult {
        match stmt {
            Stmt::Expression(expr) => self.evaluate(expr, env).map(|_| VOID),
            Stmt::Function { name, params, body } => todo!(),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => self.visit_if_statement(condition, then_branch, else_branch, clone_env(&env)),
            Stmt::Print(expr) => {
                println!("{}", self.evaluate(expr, env).map(|v| v.to_string())?);
                Ok(VOID)
            }
            Stmt::Var { token, initializer } => {
                let value = match initializer {
                    Some(expr) => self.evaluate(expr, clone_env(&env))?,
                    None => Value::Nil,
                };

                env.borrow_mut().define(token.lexeme, value);

                Ok(VOID)
            }
            Stmt::While { condition, body } => self.visit_while_statement(condition, body, env),
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
