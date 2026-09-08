use std::collections::HashMap;

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

type ResolveResult = std::result::Result<Void, ResolveError>;
const VOID_OK: ResolveResult = Ok(());

pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    scopes: Vec<HashMap<String, bool>>,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Self {
        Self {
            interpreter,
            scopes: Vec::new(),
        }
    }

    pub fn resolve_statements(&mut self, statements: &[Stmt]) -> ResolveResult {
        for stmt in statements {
            stmt.accept_visitor_mut(self)?;
        }

        VOID_OK
    }

    fn error(token: &Token, message: &str) -> ResolveResult {
        Err(ResolveError::new(token, message))
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.lexeme.clone(), false);
        }
    }

    fn define(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.last_mut()
            && let Some(initialized) = scope.get_mut(&name.lexeme)
        {
            *initialized = true
        }
    }

    fn resolve_expr(&mut self, expr: &Expr) -> ResolveResult {
        expr.accept_visitor_mut(self)
    }

    fn resolve_local(&mut self, expr: &Expr, name: &Token) {
        for (i, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(&name.lexeme) {
                self.interpreter.resolve(expr, i);
                return;
            }
        }
    }

    fn visit_binary(&mut self, left: &Expr, right: &Expr) -> ResolveResult {
        self.resolve_expr(left)?;
        self.resolve_expr(right)
    }

    fn visit_call(&mut self, callee: &Expr, arguments: &[Expr]) -> ResolveResult {
        self.resolve_expr(callee)?;

        for arg in arguments {
            self.resolve_expr(arg)?;
        }

        VOID_OK
    }

    #[cfg(feature = "conditional-op")]
    fn visit_conditional(&mut self, cond: &Expr, left: &Expr, right: &Expr) -> ResolveResult {
        self.resolve_expr(cond)?;
        self.resolve_expr(left)?;
        self.resolve_expr(right)
    }

    fn visit_grouping(&mut self, expr: &Expr) -> ResolveResult {
        self.resolve_expr(expr)
    }

    fn visit_literal(literal: &Literal) -> ResolveResult {
        VOID_OK
    }

    fn visit_logical(&mut self, left: &Expr, right: &Expr) -> ResolveResult {
        self.resolve_expr(left)?;
        self.resolve_expr(right)
    }

    fn visit_unary(&mut self, right: &Expr) -> ResolveResult {
        self.resolve_expr(right)
    }

    fn visit_variable_expr(&mut self, expr: &Expr, name: &Token) -> ResolveResult {
        if let Some(scope) = self.scopes.last()
            && let Some(initialized) = scope.get(&name.lexeme)
            && !*initialized
        {
            Self::error(name, "Can't read local variable in its own initializer.")
        } else {
            self.resolve_local(expr, name);

            VOID_OK
        }
    }

    fn visit_assign(&mut self, expr: &Expr, name: &Token, value: &Expr) -> ResolveResult {
        self.resolve_expr(value)?;
        self.resolve_local(expr, name);

        VOID_OK
    }
    
    fn visit_function_expr(&mut self, fun_expr: &FunExpr) -> ResolveResult {
        self.resolve_function(fun_expr)
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) -> ResolveResult {
        stmt.accept_visitor_mut(self)
    }

    fn visit_var_stmt(&mut self, name: &Token, initializer: &Option<Expr>) -> ResolveResult {
        self.declare(name);
        if let Some(initializer) = initializer {
            self.resolve_expr(initializer)?;
        }
        self.define(name);

        VOID_OK
    }

    fn visit_while_stmt(&mut self, condition: &Expr, body: &Stmt) -> ResolveResult {
        self.resolve_expr(condition)?;
        self.resolve_stmt(body)
    }

    fn visit_block(&mut self, statements: &[Stmt]) -> ResolveResult {
        self.begin_scope();
        self.resolve_statements(statements)?;
        self.end_scope();

        VOID_OK
    }

    fn visit_expression_stmt(&mut self, expr: &Expr) -> ResolveResult {
        self.resolve_expr(expr)
    }

    fn visit_function_stmt(&mut self, decl: &FunDecl) -> ResolveResult {
        self.declare(&decl.name);
        self.define(&decl.name);

        self.resolve_function(&decl.expr)
    }

    fn visit_if_stmt(
        &mut self,
        condition: &Expr,
        then_branch: &Stmt,
        else_branch: &Option<Box<Stmt>>,
    ) -> ResolveResult {
        self.resolve_expr(condition)?;
        self.resolve_stmt(then_branch)?;

        if let Some(else_branch) = else_branch {
            self.resolve_stmt(else_branch)?;
        }

        VOID_OK
    }

    fn visit_print_stmt(&mut self, expr: &Expr) -> ResolveResult {
        self.resolve_expr(expr)
    }

    fn visit_return_stmt(&mut self, value: &Expr) -> ResolveResult {
        self.resolve_expr(value)
    }

    fn resolve_function(&mut self, fun_expr: &FunExpr) -> ResolveResult {
        self.begin_scope();
        for param in &fun_expr.params {
            self.declare(param);
            self.define(param);
        }
        self.resolve_statements(&fun_expr.body)?;
        self.end_scope();

        VOID_OK
    }
}

impl<'a> expr::VisitorMut<ResolveResult> for Resolver<'a> {
    fn visit_expr(&mut self, expr: &crate::expr::Expr) -> ResolveResult {
        match expr {
            Expr::Binary {
                left,
                operator,
                right,
            } => self.visit_binary(left, right),
            Expr::Call {
                callee,
                paren,
                arguments,
            } => self.visit_call(callee, arguments),
            #[cfg(feature = "conditional-op")]
            Expr::Conditional { cond, left, right } => self.visit_conditional(cond, left, right),
            Expr::Grouping(expr) => self.visit_grouping(expr),
            Expr::Literal(literal) => Self::visit_literal(literal),
            Expr::Logical {
                left,
                operator,
                right,
            } => self.visit_logical(left, right),
            Expr::Unary { operator, right } => self.visit_unary(right),
            expr @ Expr::Variable(name) => self.visit_variable_expr(expr, name),
            expr @ Expr::Assign { name, value } => self.visit_assign(expr, name, value),
            #[cfg(feature = "lambda")]
            Expr::Lambda(fun_expr) => self.visit_function_expr(fun_expr),
        }
    }
}

impl<'a> stmt::VisitorMut<ResolveResult> for Resolver<'a> {
    fn visit_stmt(&mut self, stmt: &stmt::Stmt) -> ResolveResult {
        match stmt {
            Stmt::Expression(expr) => self.visit_expression_stmt(expr),
            Stmt::Function(decl) => self.visit_function_stmt(decl),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => self.visit_if_stmt(condition, then_branch, else_branch),
            Stmt::Print(expr) => self.visit_print_stmt(expr),
            Stmt::Return { keyword, value } => self.visit_return_stmt(value),
            Stmt::Var { name, initializer } => self.visit_var_stmt(name, initializer), // TODO: rename `token` to `name`
            Stmt::While { condition, body } => self.visit_while_stmt(condition, body),
            Stmt::Block(statements) => self.visit_block(statements),
        }
    }
}
