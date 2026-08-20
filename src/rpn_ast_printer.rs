use crate::expr::{Expr, Visitor};

pub struct RpnAstPrinter<'a> {
    expr: &'a Expr<'a>,
}

impl<'a> RpnAstPrinter<'a> {
    pub fn new(expr: &'a Expr) -> Self {
        Self { expr }
    }

    pub fn print(&self) -> String {
        self.visit_expr(self.expr)
    }

    fn format_unary(&self, name: &str, expr: &Expr) -> String {
        format!("{} {}", expr.accept(self), name)
    }

    fn format_binary(&self, name: &str, expr1: &Expr, expr2: &Expr) -> String {
        format!("{} {} {}", expr1.accept(self), expr2.accept(self), name)
    }

    fn format_ternary(&self, name: &str, expr1: &Expr, expr2: &Expr, expr3: &Expr) -> String {
        format!(
            "{} {} {} {}",
            expr1.accept(self),
            expr2.accept(self),
            expr3.accept(self),
            name,
        )
    }
}

impl<'a> Visitor<'a, String> for RpnAstPrinter<'a> {
    fn visit_expr(&self, expr: &Expr) -> String {
        match expr {
            Expr::Binary {
                left,
                operator,
                right,
            } => self.format_binary(operator.lexeme, left, right),
            Expr::Conditional { cond, left, right } => self.format_ternary("?:", cond, left, right),
            Expr::Grouping(expr) => expr.accept(self),
            Expr::Literal(value) => value.to_string(),
            Expr::Unary { operator, right } => self.format_unary(operator.lexeme, right),
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
        let ast_printer = RpnAstPrinter::new(&expr);

        assert_eq!(ast_printer.print(), "1.0 2.0 + 4.0 3.0 - *");
    }
}
