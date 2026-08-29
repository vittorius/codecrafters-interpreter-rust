use crate::{
    environment::{BareEnv, Env, clone_env},
    expr::{Expr, VisitorMut},
};

pub struct RpnAstPrinter<'a> {
    expr: &'a Expr<'a>,
}

impl<'a> RpnAstPrinter<'a> {
    pub fn new(expr: &'a Expr<'_>) -> Self {
        Self { expr }
    }

    pub fn print(&mut self) -> String {
        self.visit_expr(self.expr, BareEnv::new().wrapped())
    }

    fn format_unary(&mut self, name: &str, expr: &'a Expr<'_>, env: Env<'a>) -> String {
        format!("{} {}", expr.accept(self, env), name)
    }

    fn format_binary(
        &mut self,
        name: &str,
        expr1: &'a Expr<'_>,
        expr2: &'a Expr<'_>,
        env: Env<'a>,
    ) -> String {
        format!(
            "{} {} {}",
            expr1.accept(self, clone_env(&env)),
            expr2.accept(self, env),
            name
        )
    }

    fn format_ternary(
        &mut self,
        name: &str,
        expr1: &'a Expr<'_>,
        expr2: &'a Expr<'_>,
        expr3: &'a Expr<'_>,
        env: Env<'a>,
    ) -> String {
        format!(
            "{} {} {} {}",
            expr1.accept(self, clone_env(&env)),
            expr2.accept(self, clone_env(&env)),
            expr3.accept(self, env),
            name,
        )
    }

    fn format_assign(&mut self, name: &str, value: &'a Expr<'_>, env: Env<'a>) -> String {
        format!("{} {} <-", name, value.accept(self, env))
    }
}

impl<'a> VisitorMut<'a, String> for RpnAstPrinter<'a> {
    fn visit_expr(&mut self, expr: &'a Expr<'_>, env: Env<'a>) -> String {
        match expr {
            Expr::Binary {
                left,
                operator,
                right,
            } => self.format_binary(operator.lexeme, left, right, env),
            Expr::Conditional { cond, left, right } => {
                self.format_ternary("?:", cond, left, right, env)
            }
            Expr::Grouping(expr) => expr.accept(self, env),
            Expr::Literal(value) => value.to_string(),
            Expr::Unary { operator, right } => self.format_unary(operator.lexeme, right, env),
            Expr::Variable(name) => name.lexeme.to_owned(),
            Expr::Assign { name, value } => self.format_assign(name.lexeme, value, env),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::scanner::{Literal, Token, TokenType};

    use super::*;

    #[test]
    fn test_rpn_ast_printer() {
        // (1 + 2) * (4 - 3)

        let expr = Expr::Binary {
            left: Expr::Grouping(
                Expr::Binary {
                    left: Expr::Literal(Literal::Num(1.0)).boxed(),
                    operator: Token::new(TokenType::PLUS, "+", None, 1),
                    right: Expr::Literal(Literal::Num(2.0)).boxed(),
                }
                .boxed(),
            )
            .boxed(),
            operator: Token::new(TokenType::STAR, "*", None, 1),
            right: Expr::Grouping(
                Expr::Binary {
                    left: Expr::Literal(Literal::Num(4.0)).boxed(),
                    operator: Token::new(TokenType::PLUS, "-", None, 1),
                    right: Expr::Literal(Literal::Num(3.0)).boxed(),
                }
                .boxed(),
            )
            .boxed(),
        };
        let mut ast_printer = RpnAstPrinter::new(&expr);

        assert_eq!(ast_printer.print(), "1.0 2.0 + 4.0 3.0 - *");
    }

    #[test]
    fn test_assignment_expression() {
        let expr = Expr::Assign {
            name: Token::new(TokenType::IDENTIFIER, "answer", None, 1),
            value: Expr::Binary {
                left: Expr::Literal(Literal::Num(40.0)).boxed(),
                operator: Token::new(TokenType::PLUS, "+", None, 1),
                right: Expr::Literal(Literal::Num(2.0)).boxed(),
            }
            .boxed(),
        };
        let mut ast_printer = RpnAstPrinter::new(&expr);

        assert_eq!(ast_printer.print(), "answer 40.0 2.0 + <-");
    }
}
