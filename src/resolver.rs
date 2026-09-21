use std::{collections::HashMap, fmt::Display, mem};

use crate::{
    expr::{self, Expr, fun_expr::FunExpr},
    interpreter::{Interpreter, Void},
    lox,
    stmt::{self, Stmt, fun_decl::FunDecl},
    token::{Literal, Token},
};

pub struct ResolveError(String);

impl ResolveError {
    pub fn new(token: &Token, message: &str) -> Self {
        ResolveError(lox::fmt_runtime_error(token.line, message))
    }
}

impl From<ResolveError> for String {
    fn from(value: ResolveError) -> Self {
        value.0
    }
}

impl Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

enum FunctionType {
    None,
    Function,
    #[cfg(feature = "lambda")]
    Lambda,
}

type ResolutionResult = std::result::Result<Void, ResolveError>;
const VOID_OK: ResolutionResult = Ok(());

pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Self {
        Self {
            interpreter,
            scopes: Vec::new(),
            current_function: FunctionType::None,
        }
    }

    pub fn resolve_statements(&mut self, statements: &mut [Stmt]) -> ResolutionResult {
        for stmt in statements {
            stmt.accept_visitor_mut(self)?;
        }

        VOID_OK
    }

    fn error(token: &Token, message: &str) -> ResolutionResult {
        Err(ResolveError::new(token, message))
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &Token) -> ResolutionResult {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(&name.lexeme) {
                return Self::error(name, "Already a variable with this name in this scope.");
            } else {
                scope.insert(name.lexeme.clone(), false);
            }
        }

        VOID_OK
    }

    fn define(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.last_mut()
            && let Some(initialized) = scope.get_mut(&name.lexeme)
        {
            *initialized = true
        }
    }

    fn resolve_expr(&mut self, expr: &mut Expr) -> ResolutionResult {
        expr.accept_visitor_mut(self)
    }

    fn resolve_local(&mut self, name: &Token, depth: &mut Option<usize>) {
        for (i, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(&name.lexeme) {
                depth.replace(i);
                return;
            }
        }
    }

    fn visit_binary(&mut self, left: &mut Expr, right: &mut Expr) -> ResolutionResult {
        self.resolve_expr(left)?;
        self.resolve_expr(right)
    }

    fn visit_call(&mut self, callee: &mut Expr, arguments: &mut [Expr]) -> ResolutionResult {
        self.resolve_expr(callee)?;

        for arg in arguments {
            self.resolve_expr(arg)?;
        }

        VOID_OK
    }

    #[cfg(feature = "conditional-op")]
    fn visit_conditional(
        &mut self,
        cond: &mut Expr,
        left: &mut Expr,
        right: &mut Expr,
    ) -> ResolutionResult {
        self.resolve_expr(cond)?;
        self.resolve_expr(left)?;
        self.resolve_expr(right)
    }

    fn visit_grouping(&mut self, expr: &mut Expr) -> ResolutionResult {
        self.resolve_expr(expr)
    }

    fn visit_literal(_literal: &Literal) -> ResolutionResult {
        VOID_OK
    }

    fn visit_logical(&mut self, left: &mut Expr, right: &mut Expr) -> ResolutionResult {
        self.resolve_expr(left)?;
        self.resolve_expr(right)
    }

    fn visit_unary(&mut self, right: &mut Expr) -> ResolutionResult {
        self.resolve_expr(right)
    }

    fn visit_variable_expr(&mut self, name: &Token, depth: &mut Option<usize>) -> ResolutionResult {
        if let Some(scope) = self.scopes.last()
            && let Some(initialized) = scope.get(&name.lexeme)
            && !*initialized
        {
            Self::error(name, "Can't read local variable in its own initializer.")
        } else {
            self.resolve_local(name, depth);

            VOID_OK
        }
    }

    fn visit_assign(
        &mut self,
        name: &Token,
        depth: &mut Option<usize>,
        value: &mut Expr,
    ) -> ResolutionResult {
        self.resolve_expr(value)?;
        self.resolve_local(name, depth);

        VOID_OK
    }

    #[cfg(feature = "lambda")]
    fn visit_function_expr(&mut self, fun_expr: &mut FunExpr) -> ResolutionResult {
        self.resolve_function(fun_expr, FunctionType::Lambda)
    }

    fn resolve_stmt(&mut self, stmt: &mut Stmt) -> ResolutionResult {
        stmt.accept_visitor_mut(self)
    }

    fn visit_var_stmt(&mut self, name: &Token, initializer: &mut Option<Expr>) -> ResolutionResult {
        self.declare(name)?;
        if let Some(initializer) = initializer {
            self.resolve_expr(initializer)?;
        }
        self.define(name);

        VOID_OK
    }

    fn visit_while_stmt(&mut self, condition: &mut Expr, body: &mut Stmt) -> ResolutionResult {
        self.resolve_expr(condition)?;
        self.resolve_stmt(body)
    }

    fn visit_block(&mut self, statements: &mut [Stmt]) -> ResolutionResult {
        self.begin_scope();
        self.resolve_statements(statements)?;
        self.end_scope();

        VOID_OK
    }

    fn visit_expression_stmt(&mut self, expr: &mut Expr) -> ResolutionResult {
        self.resolve_expr(expr)
    }

    fn visit_function_stmt(&mut self, decl: &mut FunDecl) -> ResolutionResult {
        self.declare(&decl.name)?;
        self.define(&decl.name);

        self.resolve_function(&mut decl.expr, FunctionType::Function)
    }

    fn visit_if_stmt(
        &mut self,
        condition: &mut Expr,
        then_branch: &mut Stmt,
        else_branch: &mut Option<Box<Stmt>>,
    ) -> ResolutionResult {
        self.resolve_expr(condition)?;
        self.resolve_stmt(then_branch)?;

        if let Some(else_branch) = else_branch {
            self.resolve_stmt(else_branch)?;
        }

        VOID_OK
    }

    fn visit_print_stmt(&mut self, expr: &mut Expr) -> ResolutionResult {
        self.resolve_expr(expr)
    }

    fn visit_return_stmt(&mut self, keyword: &Token, value: &mut Expr) -> ResolutionResult {
        match self.current_function {
            FunctionType::None => Self::error(keyword, "Can't return from top-level code."),
            _ => self.resolve_expr(value),
        }
    }

    fn resolve_function(
        &mut self,
        fun_expr: &mut FunExpr,
        fun_type: FunctionType,
    ) -> ResolutionResult {
        let enclosing_function = mem::replace(&mut self.current_function, fun_type);

        self.begin_scope();
        for param in &fun_expr.params {
            self.declare(param)?;
            self.define(param);
        }
        self.resolve_statements(&mut fun_expr.body)?;
        self.end_scope();

        self.current_function = enclosing_function;

        VOID_OK
    }
}

impl<'a> expr::VisitorMut<ResolutionResult> for Resolver<'a> {
    fn visit_expr(&mut self, expr: &mut Expr) -> ResolutionResult {
        match expr {
            Expr::Binary { left, right, .. } => self.visit_binary(left, right),
            Expr::Call {
                callee, arguments, ..
            } => self.visit_call(callee, arguments),
            #[cfg(feature = "conditional-op")]
            Expr::Conditional { cond, left, right } => self.visit_conditional(cond, left, right),
            Expr::Grouping(expr) => self.visit_grouping(expr),
            Expr::Literal(literal) => Self::visit_literal(literal),
            Expr::Logical { left, right, .. } => self.visit_logical(left, right),
            Expr::Unary { right, .. } => self.visit_unary(right),
            Expr::Variable { name, depth } => self.visit_variable_expr(name, depth),
            Expr::Assign { name, depth, value } => self.visit_assign(name, depth, value),
            #[cfg(feature = "lambda")]
            Expr::Lambda(fun_expr) => self.visit_function_expr(fun_expr),
        }
    }
}

impl<'a> stmt::VisitorMut<ResolutionResult> for Resolver<'a> {
    fn visit_stmt(&mut self, stmt: &mut stmt::Stmt) -> ResolutionResult {
        match stmt {
            Stmt::Expression(expr) => self.visit_expression_stmt(expr),
            Stmt::Function(decl) => self.visit_function_stmt(decl),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => self.visit_if_stmt(condition, then_branch, else_branch),
            Stmt::Print(expr) => self.visit_print_stmt(expr),
            Stmt::Return { keyword, value } => self.visit_return_stmt(keyword, value),
            Stmt::Var { name, initializer } => self.visit_var_stmt(name, initializer),
            Stmt::While { condition, body } => self.visit_while_stmt(condition, body),
            Stmt::Block(statements) => self.visit_block(statements),
        }
    }
}
