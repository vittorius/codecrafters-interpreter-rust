#![cfg(feature = "rpn-ast-printer")]

#[cfg(feature = "lambdas")]
use crate::expr::FunExpr;
use crate::{
    expr::{Binding, Expr},
    printer::expr_visitor::Visitor,
};

#[allow(dead_code)]
pub struct RpnAstPrinter;

#[allow(dead_code, clippy::unused_self, clippy::format_push_string)]
impl RpnAstPrinter {
    pub fn print(&self, expr: &Expr) -> String {
        self.visit_expr(expr)
    }

    fn format_unary(&self, name: &str, expr: &Expr) -> String {
        format!("{} {}", expr.accept_visitor(self), name)
    }

    fn format_binary(&self, name: &str, left: &Expr, right: &Expr) -> String {
        format!(
            "{} {} {}",
            left.accept_visitor(self),
            right.accept_visitor(self),
            name
        )
    }

    fn format_set(&self, object: &Expr, name: &str, value: &Expr) -> String {
        format!(
            "{}.{} {} =",
            object.accept_visitor(self),
            name,
            value.accept_visitor(self)
        )
    }

    fn format_super(&self, method: &str) -> String {
        format!("{method} <|")
    }

    fn format_call(&self, callee: &Expr, arguments: &[Expr]) -> String {
        let mut s = String::new();
        for arg in arguments {
            s.push_str(&format!("{} ", arg.accept_visitor(self)));
        }
        s.push_str(&format!("{} ()", callee.accept_visitor(self)));
        s
    }

    #[cfg(feature = "conditional-op")]
    fn format_conditional(&self, cond: &Expr, left: &Expr, right: &Expr) -> String {
        format!(
            "{} {} {} ?:",
            cond.accept_visitor(self),
            left.accept_visitor(self),
            right.accept_visitor(self),
        )
    }

    fn format_get(&self, object: &Expr, name: &str) -> String {
        format!("{} {} .", object.accept_visitor(self), name)
    }

    fn format_assign(&self, name: &str, value: &Expr) -> String {
        format!("{} {} <-", name, value.accept_visitor(self))
    }
    #[cfg(feature = "lambdas")]
    fn format_lambda(&self, fun_expr: &FunExpr) -> String {
        let mut s = String::from("(");
        for param in &fun_expr.params {
            s.push_str(&format!("{} ", param.lexeme));
        }
        s.push_str(&format!("{})", "\\->"));
        s
    }
}

impl Visitor<String> for RpnAstPrinter {
    fn visit_expr(&self, expr: &Expr) -> String {
        match expr {
            Expr::Assign { variable, value } => self.format_assign(&variable.name.lexeme, value),
            Expr::Binary {
                left,
                operator,
                right,
            } => self.format_binary(&operator.lexeme, left, right),
            Expr::Call {
                callee, arguments, ..
            } => self.format_call(callee, arguments),
            #[cfg(feature = "conditional-op")]
            Expr::Conditional { cond, left, right } => self.format_conditional(cond, left, right),
            Expr::Get { object, name } => self.format_get(object, &name.lexeme),
            Expr::Grouping(expr) => expr.accept_visitor(self),
            #[cfg(feature = "lambdas")]
            Expr::Lambda(fun_expr) => self.format_lambda(fun_expr),
            Expr::Literal(value) => value.to_string(),
            Expr::Logical {
                left,
                operator,
                right,
            } => self.format_binary(&operator.lexeme, left, right),
            Expr::Set {
                object,
                name,
                value,
            } => self.format_set(object, &name.lexeme, value),
            Expr::Super { method, .. } => self.format_super(&method.lexeme),
            Expr::This(Binding { name, .. }) => name.lexeme.clone(),
            Expr::Unary { operator, right } => self.format_unary(&operator.lexeme, right),
            Expr::Variable(Binding { name, .. }) => name.lexeme.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::token::{Literal, Token, TokenType};

    use super::*;

    #[test]
    fn test_rpn_ast_printer() {
        // (1 + 2) * (4 - 3)

        let expr = Expr::Binary {
            left: Expr::Grouping(
                Expr::Binary {
                    left: Expr::Literal(Literal::Num(1.0)).boxed(),
                    operator: Token::new(TokenType::PLUS, "+".to_owned(), None, 1),
                    right: Expr::Literal(Literal::Num(2.0)).boxed(),
                }
                .boxed(),
            )
            .boxed(),
            operator: Token::new(TokenType::STAR, "*".to_owned(), None, 1),
            right: Expr::Grouping(
                Expr::Binary {
                    left: Expr::Literal(Literal::Num(4.0)).boxed(),
                    operator: Token::new(TokenType::PLUS, "-".to_owned(), None, 1),
                    right: Expr::Literal(Literal::Num(3.0)).boxed(),
                }
                .boxed(),
            )
            .boxed(),
        };

        let ast_printer = RpnAstPrinter;

        assert_eq!(ast_printer.print(&expr), "1.0 2.0 + 4.0 3.0 - *");
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

        let ast_printer = RpnAstPrinter;

        assert_eq!(ast_printer.print(&expr), "answer 40.0 2.0 + <-");
    }
}
