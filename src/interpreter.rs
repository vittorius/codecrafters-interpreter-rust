use std::{collections::HashMap, rc::Rc};

#[cfg(feature = "lambdas")]
use crate::expr::fun_expr::FunExpr;
use crate::{
    class::Class,
    environment::{Env, EnvShared, clone_env},
    error::RuntimeError,
    expr::{self, Expr},
    function::Function,
    native::ClockFunction,
    stmt::{self, Stmt, fun_decl::FunDecl},
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
    env: EnvShared,
}

impl Interpreter {
    pub fn new() -> Self {
        let env = Env::new().wrapped();
        env.borrow_mut()
            .define("clock".to_owned(), Value::Callable(Rc::new(ClockFunction)));
        Self { env }
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

    pub fn globals(&self) -> EnvShared {
        clone_env(&self.env)
    }

    fn execute(&self, stmt: &Stmt, env: EnvShared) -> StmtResult {
        stmt.accept_visitor_env(self, env)
    }

    pub fn execute_block(&self, statements: &[Stmt], env: EnvShared) -> StmtResult {
        for stmt in statements {
            self.execute(stmt, clone_env(&env))?;
            if env.borrow().is_returning_from_fn() {
                break;
            }
        }

        VOID_OK
    }

    fn evaluate(&self, expr: &Expr, env: EnvShared) -> ExprResult {
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

    fn visit_grouping_expr(&self, expr: &Expr, env: EnvShared) -> ExprResult {
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
        env: EnvShared,
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

    fn visit_set_expr(
        &self,
        object: &Expr,
        name: &Token,
        value: &Expr,
        env: EnvShared,
    ) -> ExprResult {
        let object = self.evaluate(object, clone_env(&env))?;

        if let Object(instance) = object {
            let value = self.evaluate(value, env)?;
            instance.borrow_mut().set(name.clone(), value.clone());

            Ok(value)
        } else {
            Self::error(name, "Only instances have fields.")
        }
    }

    fn visit_this_expr(
        &self,
        keyword: &Token,
        depth: &Option<usize>,
        env: EnvShared,
    ) -> ExprResult {
        self.lookup_variable(keyword, depth, env)
            .ok_or_else(|| unreachable!("'this' should be always defined by resolver."))
    }

    fn visit_unary_expr(&self, operator: &Token, expr: &Expr, env: EnvShared) -> ExprResult {
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
        env: EnvShared,
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
        env: EnvShared,
    ) -> ExprResult {
        let callee = self.evaluate(callee, clone_env(&env))?;

        let arguments = arguments
            .iter()
            .map(|arg| self.evaluate(arg, clone_env(&env)))
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
        env: EnvShared,
    ) -> ExprResult {
        if Self::is_truthy(&self.evaluate(cond, clone_env(&env))?) {
            self.evaluate(left, env)
        } else {
            self.evaluate(right, env)
        }
    }

    fn visit_get_expr(&self, object: &Expr, name: &Token, env: EnvShared) -> ExprResult {
        let object = self.evaluate(object, env)?;

        if let Value::Object(instance) = object {
            instance.borrow().get(name).ok_or_else(|| {
                Self::mk_error(name, &format!("Undefined property '{}'.", name.lexeme))
            })
        } else {
            Self::error(name, "Only instances have properties.")
        }
    }

    fn visit_variable_expr(
        &self,
        name: &Token,
        depth: &Option<usize>,
        env: EnvShared,
    ) -> ExprResult {
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

    fn lookup_variable(
        &self,
        name: &Token,
        depth: &Option<usize>,
        env: EnvShared,
    ) -> Option<Value> {
        if let Some(distance) = depth {
            env.borrow().get_at(*distance, &name.lexeme)
        } else {
            self.globals().borrow().get(&name.lexeme)
        }
    }

    fn visit_assign_expr(
        &self,
        name: &Token,
        depth: &Option<usize>,
        value: &Expr,
        env: EnvShared,
    ) -> ExprResult {
        let value = self.evaluate(value, clone_env(&env))?;

        if let Some(distance) = depth {
            env.borrow_mut().assign_at(*distance, name, value)
        } else {
            self.globals().borrow_mut().assign(name, value)
        }
    }

    #[cfg(feature = "lambdas")]
    fn visit_function_expr(&self, fun_expr: FunExpr, env: EnvShared) -> ExprResult {
        let function = Function::new_lambda(fun_expr, clone_env(&env));
        Ok(Value::Callable(Rc::new(function)))
    }

    fn visit_expression_stmt(&self, expr: &Expr, env: EnvShared) -> StmtResult {
        self.evaluate(expr, env).and(VOID_OK)
    }

    fn visit_function_stmt(&self, decl: FunDecl, env: EnvShared) -> StmtResult {
        let function = Function::new(decl, clone_env(&env), false);
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
        env: EnvShared,
    ) -> StmtResult {
        if Self::is_truthy(&self.evaluate(condition, clone_env(&env))?) {
            self.execute(then_branch, env)?;
        } else if let Some(else_branch) = else_branch {
            self.execute(else_branch, env)?;
        }
        VOID_OK
    }

    fn visit_print_stmt(&self, expr: &Expr, env: EnvShared) -> StmtResult {
        println!("{}", self.evaluate(expr, env).map(|v| v.to_string())?);
        VOID_OK
    }

    fn visit_return_stmt(&self, expr: &Option<Expr>, env: EnvShared) -> StmtResult {
        let return_value = if let Some(expr) = expr {
            self.evaluate(expr, clone_env(&env))?
        } else {
            Value::Nil // return; implicitly returns nil
        };
        env.borrow_mut().return_from_fn(return_value);

        VOID_OK
    }

    fn visit_variable_stmt(
        &self,
        name: &Token,
        initializer: &Option<Expr>,
        env: EnvShared,
    ) -> StmtResult {
        let value = match initializer {
            Some(expr) => self.evaluate(expr, clone_env(&env))?,
            None => Value::Nil,
        };

        env.borrow_mut().define(name.lexeme.clone(), value);

        VOID_OK
    }

    fn visit_while_stmt(&self, condition: &Expr, body: &Stmt, env: EnvShared) -> StmtResult {
        while Self::is_truthy(&self.evaluate(condition, clone_env(&env))?) {
            self.execute(body, clone_env(&env))?;
            if env.borrow().is_returning_from_fn() {
                break;
            }
        }

        VOID_OK
    }

    // Two-stage variable binding process allows references to the class inside its own methods.
    fn visit_class_stmt(&self, name: Token, methods: Vec<FunDecl>, env: EnvShared) -> StmtResult {
        // TODO: improve interfaces of all Env methods involved here to reduce the number of .clone()-s
        env.borrow_mut().define(name.lexeme.clone(), Value::Nil);

        let mut class_methods = HashMap::<String, Rc<Function>>::new();

        for method_decl in methods {
            // FIXME: avoid this cloning
            let method_name = method_decl.name.lexeme.clone();
            let function = Function::new(method_decl, clone_env(&env), &method_name == "init");
            class_methods.insert(method_name, Rc::new(function));
        }

        let class = Class::new(name.clone(), class_methods);
        env.borrow_mut()
            .assign(&name, Value::Callable(Rc::new(class)))?;

        VOID_OK
    }
}

impl expr::VisitorEnv<ExprResult> for Interpreter {
    fn visit_expr(&self, expr: &Expr, env: EnvShared) -> ExprResult {
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
    fn visit_stmt(&self, stmt: &Stmt, env: EnvShared) -> StmtResult {
        match stmt {
            Stmt::Block(statements) => {
                self.execute_block(statements, Env::with_enclosing(clone_env(&env)).wrapped())
            }
            Stmt::Class { name, methods } => {
                // TODO: cloning the entire vector here, not good.
                // Again, let's experiment with consuming Visitor for Interpreter later.
                self.visit_class_stmt(name.clone(), methods.clone(), env)
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
            } => self.visit_if_stmt(condition, then_branch, else_branch, clone_env(&env)),
            Stmt::Print(expr) => self.visit_print_stmt(expr, env),
            Stmt::Return { value, .. } => self.visit_return_stmt(value, env),
            Stmt::Var { name, initializer } => self.visit_variable_stmt(name, initializer, env),
            Stmt::While { condition, body } => self.visit_while_stmt(condition, body, env),
        }
    }
}
