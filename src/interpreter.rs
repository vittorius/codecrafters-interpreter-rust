use std::{collections::HashMap, rc::Rc};

#[cfg(feature = "lambdas")]
use crate::expr::fun_expr::FunExpr;
use crate::{
    class::Class,
    environment::Env,
    error::RuntimeError,
    expr::{self, Expr},
    function::{Function, FunctionShared},
    native::ClockFunction,
    stmt::{
        self, Stmt,
        class_decl::ClassDecl,
        fun_decl::FunDecl,
    },
    token::{self, Token, TokenType as TT},
    value::Value::{self, Callable, Object},
};

pub type Void = (); // right now, trying to follow the book, maybe remove it later
const VOID_OK: StmtResult = Ok(());

pub type Result = std::result::Result<Void, RuntimeError>;
pub type StringResult = std::result::Result<String, RuntimeError>;
type StmtResult = std::result::Result<Void, RuntimeError>;
type ExprResult = std::result::Result<Value, RuntimeError>;

pub struct Interpreter {
    env: Env,
}

impl Interpreter {
    pub fn new() -> Self {
        let env = Env::new(None);
        env.define("clock".to_owned(), Value::Callable(Rc::new(ClockFunction)));

        Self { env }
    }

    pub fn interpret(&self, statements: &[Stmt]) -> Result {
        for stmt in statements {
            self.execute(stmt, &self.env)?;
            if self.env.is_returning_from_fn() {
                break;
            }
        }

        VOID_OK
    }

    pub fn interpret_expr(&self, expr: &Expr) -> StringResult {
        self.evaluate(expr, &self.env).map(|v| v.to_string())
    }

    pub fn globals(&self) -> &Env {
        &self.env
    }

    fn execute(&self, stmt: &Stmt, env: &Env) -> StmtResult {
        stmt.accept_visitor_env(self, env)
    }

    pub fn execute_block(&self, statements: &[Stmt], env: &Env) -> StmtResult {
        for stmt in statements {
            self.execute(stmt, env)?;
            if env.is_returning_from_fn() {
                break;
            }
        }

        VOID_OK
    }

    fn evaluate(&self, expr: &Expr, env: &Env) -> ExprResult {
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

    fn mk_error(token: &Token, message: &str) -> RuntimeError {
        RuntimeError::new(token, message)
    }

    fn error(token: &Token, message: &str) -> ExprResult {
        Err(Self::mk_error(token, message))
    }

    fn visit_grouping_expr(&self, expr: &Expr, env: &Env) -> ExprResult {
        self.evaluate(expr, env)
    }

    fn visit_literal_expr(literal: &token::Literal) -> ExprResult {
        Ok(match literal {
            token::Literal::Str(s) => Value::Str(s.clone()),
            token::Literal::Num(n) => Value::Num(*n),
            token::Literal::Bool(b) => Value::Bool(*b),
            token::Literal::Nil => Value::Nil,
        })
    }

    fn visit_logical_expr(
        &self,
        left: &Expr,
        operator: &Token,
        right: &Expr,
        env: &Env,
    ) -> ExprResult {
        let left = self.evaluate(left, env)?;

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

    fn visit_set_expr(&self, object: &Expr, name: &Token, value: &Expr, env: &Env) -> ExprResult {
        let object = self.evaluate(object, env)?;

        if let Object(instance) = object {
            let value = self.evaluate(value, env)?;
            instance.borrow_mut().set(name.clone(), value.clone());

            Ok(value)
        } else {
            Self::error(name, "Only instances have fields.")
        }
    }

    fn visit_this_expr(&self, keyword: &Token, depth: &Option<usize>, env: &Env) -> ExprResult {
        self.lookup_variable(keyword, depth, env)
            .ok_or_else(|| unreachable!("'this' should be always defined by resolver."))
    }

    fn visit_unary_expr(&self, operator: &Token, expr: &Expr, env: &Env) -> ExprResult {
        let right = self.evaluate(expr, env)?;

        match (operator.token_type, right) {
            (TT::MINUS, Value::Num(n)) => Ok(Value::Num(-n)),
            (TT::MINUS, _) => Self::error(operator, "Operand must be a number."),
            (TT::BANG, val) => Ok(Value::Bool(!Self::is_truthy(&val))),
            _ => unreachable!(),
        }
    }

    fn visit_binary_expr(
        &self,
        left: &Expr,
        operator: &Token,
        right: &Expr,
        env: &Env,
    ) -> ExprResult {
        let left = self.evaluate(left, env)?;
        let right = self.evaluate(right, env)?;

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

    fn visit_call_expr(
        &self,
        callee: &Expr,
        paren: &Token,
        arguments: &[Expr],
        env: &Env,
    ) -> ExprResult {
        let callee = self.evaluate(callee, env)?;

        let arguments = arguments
            .iter()
            .map(|arg| self.evaluate(arg, env))
            .collect::<std::result::Result<Vec<_>, _>>()?;

        if let Callable(callable) = callee {
            if arguments.len() != callable.arity() {
                return Self::error(
                    paren,
                    &format!(
                        "Expected {} arguments but got {}.",
                        callable.arity(),
                        arguments.len()
                    ),
                );
            }

            callable.call(self, &arguments)
        } else {
            Self::error(paren, "Can only call functions and classes.")
        }
    }

    #[cfg(feature = "conditional-op")]
    fn visit_conditional_expr(
        &self,
        cond: &Expr,
        left: &Expr,
        right: &Expr,
        env: &Env,
    ) -> ExprResult {
        if Self::is_truthy(&self.evaluate(cond, env)?) {
            self.evaluate(left, env)
        } else {
            self.evaluate(right, env)
        }
    }

    fn visit_get_expr(&self, object: &Expr, name: &Token, env: &Env) -> ExprResult {
        let object = self.evaluate(object, env)?;

        if let Value::Object(instance) = object {
            instance.borrow().get(name).ok_or_else(|| {
                Self::mk_error(name, &format!("Undefined property '{}'.", name.lexeme))
            })
        } else {
            Self::error(name, "Only instances have properties.")
        }
    }

    fn visit_variable_expr(&self, name: &Token, depth: &Option<usize>, env: &Env) -> ExprResult {
        match self.lookup_variable(name, depth, env) {
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

    fn lookup_variable(&self, name: &Token, depth: &Option<usize>, env: &Env) -> Option<Value> {
        if let Some(distance) = depth {
            env.get_at(*distance, &name.lexeme)
        } else {
            self.globals().get(&name.lexeme)
        }
    }

    fn visit_assign_expr(
        &self,
        name: &Token,
        depth: &Option<usize>,
        value: &Expr,
        env: &Env,
    ) -> ExprResult {
        let value = self.evaluate(value, env)?;

        if let Some(distance) = depth {
            env.assign_at(*distance, name, value)
        } else {
            self.globals().assign(name, value)
        }
    }

    #[cfg(feature = "lambdas")]
    fn visit_function_expr(&self, fun_expr: FunExpr, env: &Env) -> ExprResult {
        let function = Function::new_lambda(fun_expr, Env::clone(env));
        Ok(Value::Callable(Rc::new(function)))
    }

    fn visit_expression_stmt(&self, expr: &Expr, env: &Env) -> StmtResult {
        self.evaluate(expr, env).and(VOID_OK)
    }

    fn visit_function_stmt(&self, decl: FunDecl, env: &Env) -> StmtResult {
        let function = Function::new(Rc::new(decl), Env::clone(env), false);
        env.define(
            function.name().to_owned(),
            Value::Callable(Rc::new(function)),
        );

        VOID_OK
    }

    fn visit_if_stmt(
        &self,
        condition: &Expr,
        then_branch: &Stmt,
        else_branch: &Option<Box<Stmt>>,
        env: &Env,
    ) -> StmtResult {
        if Self::is_truthy(&self.evaluate(condition, env)?) {
            self.execute(then_branch, env)?;
        } else if let Some(else_branch) = else_branch {
            self.execute(else_branch, env)?;
        }
        VOID_OK
    }

    fn visit_print_stmt(&self, expr: &Expr, env: &Env) -> StmtResult {
        println!("{}", self.evaluate(expr, env).map(|v| v.to_string())?);
        VOID_OK
    }

    fn visit_return_stmt(&self, expr: &Option<Expr>, env: &Env) -> StmtResult {
        let return_value = if let Some(expr) = expr {
            self.evaluate(expr, env)?
        } else {
            Value::Nil // return; implicitly returns nil
        };
        env.set_return_from_fn(return_value);

        VOID_OK
    }

    fn visit_variable_stmt(
        &self,
        name: &Token,
        initializer: &Option<Expr>,
        env: &Env,
    ) -> StmtResult {
        let value = match initializer {
            Some(expr) => self.evaluate(expr, env)?,
            None => Value::Nil,
        };

        env.define(name.lexeme.clone(), value);

        VOID_OK
    }

    fn visit_while_stmt(&self, condition: &Expr, body: &Stmt, env: &Env) -> StmtResult {
        while Self::is_truthy(&self.evaluate(condition, env)?) {
            self.execute(body, env)?;
            if env.is_returning_from_fn() {
                break;
            }
        }

        VOID_OK
    }

    // Two-stage variable binding process allows references to the class inside its own methods.
    fn visit_class_stmt(&self, class_decl: ClassDecl, env: &Env) -> StmtResult {
        env.define(class_decl.name.lexeme.clone(), Value::Nil);

        let mut class_methods = HashMap::<String, FunctionShared>::new();

        for method_decl in class_decl.methods {
            let is_initializer = method_decl.name.lexeme == "init";
            let function = Function::new(Rc::new(method_decl), Env::clone(env), is_initializer);
            class_methods.insert(function.name().to_owned(), Rc::new(function));
        }

        let class = Class::new(class_decl.name.clone(), class_methods);
        env.assign(&class_decl.name, Value::Callable(Rc::new(class)))?;

        VOID_OK
    }
}

impl expr::VisitorEnv<ExprResult> for Interpreter {
    fn visit_expr(&self, expr: &Expr, env: &Env) -> ExprResult {
        match expr {
            Expr::Assign { name, depth, value } => self.visit_assign_expr(name, depth, value, env),
            Expr::Binary {
                left,
                operator,
                right,
            } => self.visit_binary_expr(left, operator, right, env),
            Expr::Call {
                callee,
                paren,
                arguments,
            } => self.visit_call_expr(callee, paren, arguments, env),
            #[cfg(feature = "conditional-op")]
            Expr::Conditional { cond, left, right } => {
                self.visit_conditional_expr(cond, left, right, env)
            }
            Expr::Get { object, name } => self.visit_get_expr(object, name, env),
            Expr::Grouping(expr) => self.visit_grouping_expr(expr, env),
            #[cfg(feature = "lambdas")]
            Expr::Lambda(fun_expr) => self.visit_function_expr(fun_expr.clone(), env),
            Expr::Literal(literal) => Self::visit_literal_expr(literal),
            Expr::Logical {
                left,
                operator,
                right,
            } => self.visit_logical_expr(left, operator, right, env),
            Expr::Set {
                object,
                name,
                value,
            } => self.visit_set_expr(object, name, value, env),
            Expr::This { keyword, depth } => self.visit_this_expr(keyword, depth, env),
            Expr::Unary { operator, right } => self.visit_unary_expr(operator, right, env),
            Expr::Variable { name, depth } => self.visit_variable_expr(name, depth, env),
        }
    }
}

impl stmt::VisitorEnv<StmtResult> for Interpreter {
    fn visit_stmt(&self, stmt: &Stmt, env: &Env) -> StmtResult {
        match stmt {
            Stmt::Block(statements) => self.execute_block(statements, &Env::new(Some(env))),
            Stmt::Class(class_decl) => {
                // A consuming Visitor for Interpreter would lead to either cloning large parts of the AST
                // every time a code block is executed (a function call, a while loop).
                self.visit_class_stmt(class_decl.clone(), env)
            }
            Stmt::Expression(expr) => self.visit_expression_stmt(expr, env),
            // Cloning function decl here prevents from using Expr pointers ("references") as keys in local variable lookup resolution.
            // On the other hand, we have to clone function decl to make it live inside the environment and outlive the interpreter
            // invocations with new source code inputs in order for REPL to work.
            Stmt::Function(decl) => self.visit_function_stmt(decl.clone(), env),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => self.visit_if_stmt(condition, then_branch, else_branch, env),
            Stmt::Print(expr) => self.visit_print_stmt(expr, env),
            Stmt::Return { value, .. } => self.visit_return_stmt(value, env),
            Stmt::Var { name, initializer } => self.visit_variable_stmt(name, initializer, env),
            Stmt::While { condition, body } => self.visit_while_stmt(condition, body, env),
        }
    }
}
