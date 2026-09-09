use std::{collections::HashMap, rc::Rc};

#[cfg(feature = "lambda")]
use crate::expr::fun_expr::FunExpr;
use crate::{
    environment::{BareEnv, Env, clone_env},
    error::RuntimeError,
    expr::{self, Expr},
    function::Function,
    native::ClockFunction,
    stmt::{self, Stmt, fun_decl::FunDecl},
    token::{self, Token, TokenType as TT},
    value::Value::{self, Callable},
};

pub type Void = (); // right now, trying to follow the book, maybe remove it later
const VOID_OK: StmtResult = Ok(());

pub type Result = std::result::Result<Void, RuntimeError>;
pub type StringResult = std::result::Result<String, RuntimeError>;
type StmtResult = std::result::Result<Void, RuntimeError>;
type ExprResult = std::result::Result<Value, RuntimeError>;

pub struct Interpreter {
    env: Env,
    locals: HashMap<*const Expr, usize>,
}

impl Interpreter {
    pub fn new() -> Self {
        let env = BareEnv::new().wrapped();
        env.borrow_mut()
            .define("clock".to_owned(), Value::Callable(Rc::new(ClockFunction)));
        Self {
            env,
            locals: HashMap::new(),
        }
    }

    pub fn interpret(&self, statements: &[Stmt]) -> Result {
        for stmt in statements {
            self.execute(stmt, clone_env(&self.env))?;
            if self.env.borrow().is_returning_from_fn() {
                break;
            }
        }

        VOID_OK
    }

    pub fn interpret_expr(&self, expr: &Expr) -> StringResult {
        self.evaluate(expr, clone_env(&self.env))
            .map(|v| v.to_string())
    }

    pub fn globals(&self) -> Env {
        clone_env(&self.env)
    }

    pub fn resolve(&mut self, expr: &Expr, depth: usize) {
        self.locals.insert(expr as *const Expr, depth);
    }

    fn execute(&self, stmt: &Stmt, env: Env) -> StmtResult {
        stmt.accept_visitor_env(self, env)
    }

    pub fn execute_block(&self, statements: &[Stmt], env: Env) -> StmtResult {
        for stmt in statements {
            self.execute(stmt, clone_env(&env))?;
            if env.borrow().is_returning_from_fn() {
                break;
            }
        }

        VOID_OK
    }

    fn evaluate(&self, expr: &Expr, env: Env) -> ExprResult {
        expr.accept_visitor_env(self, env)
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

    fn error(token: &Token, message: &str) -> ExprResult {
        Err(RuntimeError::new(token, message))
    }

    fn visit_grouping(&self, expr: &Expr, env: Env) -> ExprResult {
        self.evaluate(expr, env)
    }

    fn visit_literal(literal: &token::Literal) -> ExprResult {
        Ok(match literal {
            token::Literal::Str(s) => Value::Str(s.clone()),
            token::Literal::Num(n) => Value::Num(*n),
            token::Literal::Bool(b) => Value::Bool(*b),
            token::Literal::Nil => Value::Nil,
        })
    }

    fn visit_logical(&self, left: &Expr, operator: &Token, right: &Expr, env: Env) -> ExprResult {
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

    fn visit_unary(&self, operator: &Token, expr: &Expr, env: Env) -> ExprResult {
        let right = self.evaluate(expr, env)?;

        match (operator.token_type, right) {
            (TT::MINUS, Value::Num(n)) => Ok(Value::Num(-n)),
            (TT::MINUS, _) => Self::error(operator, "Operand must be a number."),
            (TT::BANG, val) => Ok(Value::Bool(!Self::is_truthy(&val))),
            _ => unreachable!(),
        }
    }

    fn visit_binary(&self, left: &Expr, operator: &Token, right: &Expr, env: Env) -> ExprResult {
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
            #[cfg(feature = "comma-op")]
            (TT::COMMA, _, r) => Ok(r), // discard left and return right
            _ => unreachable!("Invalid binary operation."),
        }
    }

    fn visit_call(&self, callee: &Expr, paren: &Token, arguments: &[Expr], env: Env) -> ExprResult {
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

            function.call(self, &arguments, env)
        } else {
            Self::error(paren, "Can only call functions and classes.")
        }
    }

    #[cfg(feature = "conditional-op")]
    fn visit_conditional(&self, cond: &Expr, left: &Expr, right: &Expr, env: Env) -> ExprResult {
        if Self::is_truthy(&self.evaluate(cond, clone_env(&env))?) {
            self.evaluate(left, env)
        } else {
            self.evaluate(right, env)
        }
    }

    fn visit_variable_expr(&self, expr: &Expr, name: &Token, env: Env) -> ExprResult {
        match self.lookup_variable(expr, name, env) {
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

    fn lookup_variable(&self, expr: &Expr, name: &Token, env: Env) -> Option<Value> {
        if let Some(distance) = self.locals.get(&(expr as *const Expr)) {
            env.borrow().get_at(*distance, name)
        } else {
            self.globals().borrow().get(name)
        }
    }

    fn visit_assign(&self, name: &Token, value: Value, env: Env) -> ExprResult {
        env.borrow_mut().assign(name, value)
    }

    #[cfg(feature = "lambda")]
    fn visit_function_expr(&self, fun_expr: FunExpr, env: Env) -> ExprResult {
        let function = Function::new_lambda(fun_expr, clone_env(&env));
        Ok(Value::Callable(Rc::new(function)))
    }

    fn visit_expression_stmt(&self, expr: &Expr, env: Env) -> StmtResult {
        self.evaluate(expr, env).and(VOID_OK)
    }

    fn visit_function_stmt(&self, decl: FunDecl, env: Env) -> StmtResult {
        let function = Function::new(decl, clone_env(&env));
        env.borrow_mut().define(
            function
                .name()
                .expect("A regular function always has a name")
                .to_owned(),
            Value::Callable(Rc::new(function)),
        );

        VOID_OK
    }

    fn visit_if_stmt(
        &self,
        condition: &Expr,
        then_branch: &Stmt,
        else_branch: &Option<Box<Stmt>>,
        env: Env,
    ) -> StmtResult {
        if Self::is_truthy(&self.evaluate(condition, clone_env(&env))?) {
            self.execute(then_branch, env)?;
        } else if let Some(else_branch) = else_branch {
            self.execute(else_branch, env)?;
        }
        VOID_OK
    }

    fn visit_print_stmt(&self, expr: &Expr, env: Env) -> StmtResult {
        println!("{}", self.evaluate(expr, env).map(|v| v.to_string())?);
        VOID_OK
    }

    fn visit_return_stmt(&self, expr: &Expr, env: Env) -> StmtResult {
        let value = self.evaluate(expr, clone_env(&env))?;
        env.borrow_mut().return_from_fn(value);

        VOID_OK
    }

    fn visit_variable_stmt(
        &self,
        name: &Token,
        initializer: &Option<Expr>,
        env: Env,
    ) -> StmtResult {
        let value = match initializer {
            Some(expr) => self.evaluate(expr, clone_env(&env))?,
            None => Value::Nil,
        };

        env.borrow_mut().define(name.lexeme.clone(), value);

        VOID_OK
    }

    fn visit_while_stmt(&self, condition: &Expr, body: &Stmt, env: Env) -> StmtResult {
        while Self::is_truthy(&self.evaluate(condition, clone_env(&env))?) {
            self.execute(body, clone_env(&env))?;
            if env.borrow().is_returning_from_fn() {
                break;
            }
        }

        VOID_OK
    }
}

impl expr::VisitorEnv<ExprResult> for Interpreter {
    fn visit_expr(&self, expr: &Expr, env: Env) -> ExprResult {
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
            #[cfg(feature = "conditional-op")]
            Expr::Conditional { cond, left, right } => {
                self.visit_conditional(cond, left, right, env)
            }
            Expr::Grouping(expr) => self.visit_grouping(expr, env),
            Expr::Literal(literal) => Self::visit_literal(literal),
            Expr::Logical {
                left,
                operator,
                right,
            } => self.visit_logical(left, operator, right, env),
            Expr::Unary { operator, right } => self.visit_unary(operator, right, env),
            expr @ Expr::Variable(name) => self.visit_variable_expr(expr, name, env),
            Expr::Assign { name, value } => {
                let value = self.evaluate(value, clone_env(&env))?;
                self.visit_assign(name, value, env)
            }
            #[cfg(feature = "lambda")]
            Expr::Lambda(fun_expr) => self.visit_function_expr(fun_expr.clone(), env),
        }
    }
}

impl stmt::VisitorEnv<StmtResult> for Interpreter {
    fn visit_stmt(&self, stmt: &Stmt, env: Env) -> StmtResult {
        match stmt {
            Stmt::Expression(expr) => self.visit_expression_stmt(expr, env),
            Stmt::Function(decl) => self.visit_function_stmt(decl.clone(), env),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => self.visit_if_stmt(condition, then_branch, else_branch, clone_env(&env)),
            Stmt::Print(expr) => self.visit_print_stmt(expr, env),
            Stmt::Return { keyword, value } => self.visit_return_stmt(value, env),
            Stmt::Var { name, initializer } => self.visit_variable_stmt(name, initializer, env),
            Stmt::While { condition, body } => self.visit_while_stmt(condition, body, env),
            Stmt::Block(statements) => self.execute_block(
                statements,
                BareEnv::with_enclosing(clone_env(&env)).wrapped(),
            ),
        }
    }
}
