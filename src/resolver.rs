use std::{fmt::Display, mem};

use crate::{
    expr::{self, Expr, fun_expr::FunExpr},
    interpreter::Void,
    lox,
    resolver::scope::Scope,
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
    #[cfg(feature = "lambdas")]
    Lambda,
}

type ResolutionResult = std::result::Result<Void, ResolveError>;
const VOID_OK: ResolutionResult = Ok(());

mod scope;

pub struct Resolver<'a> {
    scopes: Vec<Scope<'a>>, // as a stack
    current_function: FunctionType,
}

impl<'a> Resolver<'a> {
    pub fn new() -> Self {
        Self {
            scopes: Vec::new(),
            current_function: FunctionType::None,
        }
    }

    pub fn resolve(&mut self, statements: &'a mut [Stmt]) -> ResolutionResult {
        #[cfg(feature = "err-unused-vars")]
        self.begin_scope();

        self.resolve_statements(statements)?;

        #[cfg(feature = "err-unused-vars")]
        self.end_scope()?;

        VOID_OK
    }

    fn resolve_statements(&mut self, statements: &'a mut [Stmt]) -> ResolutionResult {
        for stmt in statements {
            stmt.accept_visitor_mut(self)?;
        }

        VOID_OK
    }

    fn error(token: &Token, message: &str) -> ResolutionResult {
        Err(ResolveError::new(token, message))
    }

    fn begin_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    fn end_scope(&mut self) -> ResolutionResult {
        #[cfg(feature = "err-unused-vars")]
        {
            let last_scope = self.scopes.pop();
            if let Some(last_scope) = last_scope
                && let Some(token) = last_scope.first_unused()
            {
                return Self::error(token, "Unused variable");
            }
        }

        #[cfg(not(feature = "err-unused-vars"))]
        self.scopes.pop();

        VOID_OK
    }

    fn last_scope_mut(&mut self) -> Option<&mut Scope<'a>> {
        self.scopes.last_mut()
    }

    fn declare(&mut self, name: &'a Token) -> ResolutionResult {
        if let Some(scope) = self.last_scope_mut() {
            if scope.is_declared(name) {
                return Self::error(name, "Already a variable with this name in this scope.");
            } else {
                scope.declare(name);
            }
        }

        VOID_OK
    }

    fn define(&mut self, name: &'a Token) {
        if let Some(scope) = self.last_scope_mut()
            && scope.is_declared(name)
        {
            scope.define(name);
        }
    }

    fn resolve_expr(&mut self, expr: &'a mut Expr) -> ResolutionResult {
        expr.accept_visitor_mut(self)
    }

    fn resolve_local(&mut self, name: &Token, depth: &mut Option<usize>, is_read: bool) {
        for (i, scope) in self.scopes.iter_mut().rev().enumerate() {
            if scope.is_declared(name) {
                #[cfg(feature = "err-unused-vars")]
                if is_read {
                    scope.mark_used(name);
                }

                #[cfg(not(feature = "err-unused-vars"))]
                let _ = is_read;

                depth.replace(i);
                return;
            }
        }
    }

    fn visit_binary_expr(&mut self, left: &'a mut Expr, right: &'a mut Expr) -> ResolutionResult {
        self.resolve_expr(left)?;
        self.resolve_expr(right)
    }

    fn visit_call_expr(&mut self, callee: &'a mut Expr, arguments: &'a mut [Expr]) -> ResolutionResult {
        self.resolve_expr(callee)?;

        for arg in arguments {
            self.resolve_expr(arg)?;
        }

        VOID_OK
    }

    #[cfg(feature = "conditional-op")]
    fn visit_conditional_expr(
        &mut self,
        cond: &'a mut Expr,
        left: &'a mut Expr,
        right: &'a mut Expr,
    ) -> ResolutionResult {
        self.resolve_expr(cond)?;
        self.resolve_expr(left)?;
        self.resolve_expr(right)
    }

    fn visit_grouping_expr(&mut self, expr: &'a mut Expr) -> ResolutionResult {
        self.resolve_expr(expr)
    }

    fn visit_literal_expr(_literal: &Literal) -> ResolutionResult {
        VOID_OK
    }

    fn visit_logical_expr(&mut self, left: &'a mut Expr, right: &'a mut Expr) -> ResolutionResult {
        self.resolve_expr(left)?;
        self.resolve_expr(right)
    }

    fn visit_unary_expr(&mut self, right: &'a mut Expr) -> ResolutionResult {
        self.resolve_expr(right)
    }

    fn visit_variable_expr(
        &mut self,
        name: &'a Token,
        depth: &mut Option<usize>,
    ) -> ResolutionResult {
        if let Some(scope) = self.scopes.last()
            && scope.is_declared(name)
            && !scope.is_defined(name)
        {
            return Self::error(name, "Can't read local variable in its own initializer.");
        }

        self.resolve_local(name, depth, true);

        VOID_OK
    }

    fn visit_assign_expr(
        &mut self,
        name: &Token,
        depth: &mut Option<usize>,
        value: &'a mut Expr,
    ) -> ResolutionResult {
        self.resolve_expr(value)?;
        self.resolve_local(name, depth, false);

        VOID_OK
    }

    #[cfg(feature = "lambdas")]
    fn visit_function_expr(&mut self, fun_expr: &'a mut FunExpr) -> ResolutionResult {
        self.resolve_function(fun_expr, FunctionType::Lambda)
    }

    fn resolve_stmt(&mut self, stmt: &'a mut Stmt) -> ResolutionResult {
        stmt.accept_visitor_mut(self)
    }

    fn visit_variable_stmt(
        &mut self,
        name: &'a Token,
        initializer: &'a mut Option<Expr>,
    ) -> ResolutionResult {
        self.declare(name)?;
        if let Some(initializer) = initializer {
            self.resolve_expr(initializer)?;
        }
        self.define(name);

        VOID_OK
    }

    fn visit_while_stmt(
        &mut self,
        condition: &'a mut Expr,
        body: &'a mut Stmt,
    ) -> ResolutionResult {
        self.resolve_expr(condition)?;
        self.resolve_stmt(body)
    }

    fn visit_block(&mut self, statements: &'a mut [Stmt]) -> ResolutionResult {
        self.begin_scope();
        self.resolve_statements(statements)?;
        self.end_scope()?;

        VOID_OK
    }

    fn visit_class_stmt(&mut self, name: &'a Token) -> ResolutionResult {
        self.declare(name)?;
        self.define(name);

        VOID_OK
    }

    fn visit_expression_stmt(&mut self, expr: &'a mut Expr) -> ResolutionResult {
        self.resolve_expr(expr)
    }

    fn visit_function_stmt(&mut self, decl: &'a mut FunDecl) -> ResolutionResult {
        self.declare(&decl.name)?;
        self.define(&decl.name);

        self.resolve_function(&mut decl.expr, FunctionType::Function)
    }

    fn visit_if_stmt(
        &mut self,
        condition: &'a mut Expr,
        then_branch: &'a mut Stmt,
        else_branch: &'a mut Option<Box<Stmt>>,
    ) -> ResolutionResult {
        self.resolve_expr(condition)?;
        self.resolve_stmt(then_branch)?;

        if let Some(else_branch) = else_branch {
            self.resolve_stmt(else_branch)?;
        }

        VOID_OK
    }

    fn visit_print_stmt(&mut self, expr: &'a mut Expr) -> ResolutionResult {
        self.resolve_expr(expr)
    }

    fn visit_return_stmt(&mut self, keyword: &Token, value: &'a mut Expr) -> ResolutionResult {
        match self.current_function {
            FunctionType::None => Self::error(keyword, "Can't return from top-level code."),
            _ => self.resolve_expr(value),
        }
    }

    fn resolve_function(
        &mut self,
        fun_expr: &'a mut FunExpr,
        fun_type: FunctionType,
    ) -> ResolutionResult {
        let enclosing_function = mem::replace(&mut self.current_function, fun_type);

        self.begin_scope();
        for param in &fun_expr.params {
            self.declare(param)?;
            self.define(param);
        }
        self.resolve_statements(&mut fun_expr.body)?;
        self.end_scope()?;

        self.current_function = enclosing_function;

        VOID_OK
    }
}

impl<'a> expr::VisitorMut<'a, ResolutionResult> for Resolver<'a> {
    fn visit_expr(&mut self, expr: &'a mut Expr) -> ResolutionResult {
        match expr {
            Expr::Assign { name, depth, value } => self.visit_assign_expr(name, depth, value),
            Expr::Binary { left, right, .. } => self.visit_binary_expr(left, right),
            Expr::Call {
                callee, arguments, ..
            } => self.visit_call_expr(callee, arguments),
            #[cfg(feature = "conditional-op")]
            Expr::Conditional { cond, left, right } => self.visit_conditional_expr(cond, left, right),
            Expr::Grouping(expr) => self.visit_grouping_expr(expr),
            #[cfg(feature = "lambdas")]
            Expr::Lambda(fun_expr) => self.visit_function_expr(fun_expr),
            Expr::Literal(literal) => Self::visit_literal_expr(literal),
            Expr::Logical { left, right, .. } => self.visit_logical_expr(left, right),
            Expr::Unary { right, .. } => self.visit_unary_expr(right),
            Expr::Variable { name, depth } => self.visit_variable_expr(name, depth),
        }
    }
}

impl<'a> stmt::VisitorMut<'a, ResolutionResult> for Resolver<'a> {
    fn visit_stmt(&mut self, stmt: &'a mut stmt::Stmt) -> ResolutionResult {
        match stmt {
            Stmt::Block(statements) => self.visit_block(statements),
            Stmt::Class { name, .. } => self.visit_class_stmt(name),
            Stmt::Expression(expr) => self.visit_expression_stmt(expr),
            Stmt::Function(decl) => self.visit_function_stmt(decl),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => self.visit_if_stmt(condition, then_branch, else_branch),
            Stmt::Print(expr) => self.visit_print_stmt(expr),
            Stmt::Return { keyword, value } => self.visit_return_stmt(keyword, value),
            Stmt::Var { name, initializer } => self.visit_variable_stmt(name, initializer),
            Stmt::While { condition, body } => self.visit_while_stmt(condition, body),
        }
    }
}
