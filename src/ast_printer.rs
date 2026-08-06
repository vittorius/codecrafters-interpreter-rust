use crate::expr::{Expr, Visitor};

pub struct AstPrinter<'a> {
    expr: &'a Expr,
}

impl<'a> AstPrinter<'a> {
    pub fn new(expr: &'a Expr) -> Self {
        Self { expr }
    }

    pub fn print(&self) -> String {
        self.visit_expr(self.expr)
    }

    fn parenthesize_unary(&self, name: &str, expr: &Expr) -> String {
        format!("({} {})", name, expr.accept(self))
    }

    fn parenthesize_binary(&self, name: &str, expr1: &Expr, expr2: &Expr) -> String {
        format!("({} {} {})", name, expr1.accept(self), expr2.accept(self))
    }
}

impl<'a> Visitor<String> for AstPrinter<'a> {
    fn visit_expr(&self, expr: &Expr) -> String {
        match expr {
            Expr::Binary {
                left,
                operator,
                right,
            } => self.parenthesize_binary(&operator.lexeme, left, right),
            Expr::Grouping { expr } => self.parenthesize_unary("group", expr),
            Expr::Literal { value } => match value {
                Some(value) => value.to_string(),
                None => "nil".to_owned(),
            },
            Expr::Unary { operator, right } => self.parenthesize_unary(&operator.lexeme, right),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::scanner::{Literal, Token, TokenType};

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
            left: Box::new(Expr::Unary {
                operator: Token::new(TokenType::MINUS, "-".to_owned(), None, 1),
                right: Box::new(Expr::Literal {
                    value: Some(Literal::Num(123.0)),
                }),
            }),
            operator: Token::new(TokenType::STAR, "*".to_owned(), None, 1),
            right: Box::new(Expr::Grouping {
                expr: Box::new(Expr::Literal {
                    value: Some(Literal::Num(45.67)),
                }),
            }),
        };

        let ast_printer = AstPrinter::new(&expr);

        assert_eq!(ast_printer.print(), "(* (- 123.0) (group 45.67))");
    }
}
