#[cfg(feature = "lambdas")]
use crate::expr::FunExpr;
use crate::{
    expr::{Binding, Expr},
    printer::expr_visitor::Visitor,
};

pub struct AstPrinter;

impl AstPrinter {
    pub fn print(&self, expr: &Expr) -> String {
        self.visit_expr(expr)
    }

    fn parenthesize_unary(&self, name: &str, expr: &Expr) -> String {
        format!("({} {})", name, expr.accept_visitor(self))
    }

    fn parenthesize_binary(&self, name: &str, left: &Expr, right: &Expr) -> String {
        format!(
            "({} {} {})",
            name,
            left.accept_visitor(self),
            right.accept_visitor(self)
        )
    }

    fn parenthesize_set(&self, object: &Expr, name: &str, value: &Expr) -> String {
        format!(
            "(= {}.{} {})",
            object.accept_visitor(self),
            name,
            value.accept_visitor(self)
        )
    }

    fn parenthesize_super(&self, method: &str) -> String {
        format!("(<| {method})")
    }

    fn parenthesize_call(&self, callee: &Expr, arguments: &[Expr]) -> String {
        let mut s = format!("({}", callee.accept_visitor(self));
        for arg in arguments {
            s.push_str(&format!(" {}", arg.accept_visitor(self)));
        }
        s.push(')');
        s
    }

    #[cfg(feature = "conditional-op")]
    fn parenthesize_ternary(&self, cond: &Expr, left: &Expr, right: &Expr) -> String {
        format!(
            "(?: {} {} {})",
            cond.accept_visitor(self),
            left.accept_visitor(self),
            right.accept_visitor(self)
        )
    }

    fn parenthesize_get(&self, object: &Expr, name: &str) -> String {
        format!("(. {} {})", object.accept_visitor(self), name)
    }

    fn parenthesize_assign(&self, name: &str, value: &Expr) -> String {
        format!("(<- {} {})", name, value.accept_visitor(self))
    }

    #[cfg(feature = "lambdas")]
    fn parenthesize_lambda(&self, fun_expr: &FunExpr) -> String {
        let mut s = String::from("lambda");
        for param in fun_expr.params.iter() {
            s.push_str(&format!(" {}", param.lexeme));
        }
        s.push(')');
        s
    }
}

impl Visitor<String> for AstPrinter {
    fn visit_expr(&self, expr: &Expr) -> String {
        match expr {
            Expr::Assign {
                variable, value, ..
            } => self.parenthesize_assign(&variable.name.lexeme, value),
            Expr::Binary {
                left,
                operator,
                right,
            } => self.parenthesize_binary(&operator.lexeme, left, right),
            Expr::Call {
                callee, arguments, ..
            } => self.parenthesize_call(callee, arguments),
            #[cfg(feature = "conditional-op")]
            Expr::Conditional { cond, left, right } => self.parenthesize_ternary(cond, left, right),
            Expr::Get { object, name } => self.parenthesize_get(object, &name.lexeme),
            Expr::Grouping(expr) => self.parenthesize_unary("group", expr),
            #[cfg(feature = "lambdas")]
            Expr::Lambda(fun_expr) => self.parenthesize_lambda(fun_expr),
            Expr::Literal(value) => value.to_string(),
            Expr::Logical {
                left,
                operator,
                right,
            } => self.parenthesize_binary(&operator.lexeme, left, right),
            Expr::Set {
                object,
                name,
                value,
            } => self.parenthesize_set(object, &name.lexeme, value),
            Expr::Super { method, .. } => self.parenthesize_super(&method.lexeme),
            Expr::This(Binding { name, .. }) => name.lexeme.clone(),
            Expr::Unary { operator, right } => self.parenthesize_unary(&operator.lexeme, right),
            Expr::Variable(Binding { name, .. }) => name.lexeme.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::token::{Literal, Token, TokenType};

    use super::*;

    #[test]
    fn test_ast_printer() {
        // Expr expression = new Expr.Binary(
        //   new Expr.Unary(
        //     new Token(TokenType.MINUS, "-", null, 1),
        //     new Expr.Literal(123)),
        //   new Token(TokenType.STAR, "*", null, 1),
        //   new Expr.Grouping(
        //     new Expr.Literal(45.67)));

        let expr = Expr::Binary {
            left: Expr::Unary {
                operator: Token::new(TokenType::MINUS, "-".to_owned(), None, 1),
                right: Expr::Literal(Literal::Num(123.0)).boxed(),
            }
            .boxed(),
            operator: Token::new(TokenType::STAR, "*".to_owned(), None, 1),
            right: Expr::Grouping(Expr::Literal(Literal::Num(45.67)).boxed()).boxed(),
        };

        let ast_printer = AstPrinter;

        assert_eq!(ast_printer.print(&expr), "(* (- 123.0) (group 45.67))");
    }

    #[test]
    fn test_assignment_expression() {
        let expr = Expr::Assign {
            variable: Binding {
                name: Token::new(TokenType::IDENTIFIER, "answer".to_owned(), None, 1),
                depth: None,
            },
            value: Expr::Binary {
                left: Expr::Literal(Literal::Num(40.0)).boxed(),
                operator: Token::new(TokenType::PLUS, "+".to_owned(), None, 1),
                right: Expr::Literal(Literal::Num(2.0)).boxed(),
            }
            .boxed(),
        };

        let ast_printer = AstPrinter;

        assert_eq!(ast_printer.print(&expr), "(<- answer (+ 40.0 2.0))");
    }
}
