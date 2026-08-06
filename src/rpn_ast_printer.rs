use crate::expr::{Expr, Visitor};

pub struct RpnAstPrinter<'a> {
    expr: &'a Expr,
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
}

impl<'a> Visitor<String> for RpnAstPrinter<'a> {
    fn visit_expr(&self, expr: &Expr) -> String {
        match expr {
            Expr::Binary {
                left,
                operator,
                right,
            } => self.format_binary(&operator.lexeme, left, right),
            Expr::Grouping { expr } => expr.accept(self),
            Expr::Literal { value } => match value {
                Some(value) => value.to_string(),
                None => "nil".to_owned(),
            },
            Expr::Unary { operator, right } => self.format_unary(&operator.lexeme, right),
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
            left: Expr::Grouping {
                expr: Expr::Binary {
                    left: Expr::Literal {
                        value: Some(Literal::Num(1.0)),
                    }
                    .boxed(),
                    operator: Token::new(TokenType::PLUS, "+".to_owned(), None, 1),
                    right: Expr::Literal {
                        value: Some(Literal::Num(2.0)),
                    }
                    .boxed(),
                }
                .boxed(),
            }
            .boxed(),
            operator: Token::new(TokenType::STAR, "*".to_owned(), None, 1),
            right: Expr::Grouping {
                expr: Expr::Binary {
                    left: Expr::Literal {
                        value: Some(Literal::Num(4.0)),
                    }
                    .boxed(),
                    operator: Token::new(TokenType::PLUS, "-".to_owned(), None, 1),
                    right: Expr::Literal {
                        value: Some(Literal::Num(3.0)),
                    }
                    .boxed(),
                }
                .boxed(),
            }
            .boxed(),
        };
        let ast_printer = RpnAstPrinter::new(&expr);

        assert_eq!(ast_printer.print(), "1.0 2.0 + 4.0 3.0 - *");
    }
}
