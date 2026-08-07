use crate::expr::{ExprRef, VisitorRef};

pub struct AstPrinterRef<'a> {
    expr: &'a ExprRef<'a>,
}

impl<'a> AstPrinterRef<'a> {
    pub fn new(expr: &'a ExprRef) -> Self {
        Self { expr }
    }

    pub fn print(&self) -> String {
        self.visit_expr(self.expr)
    }

    fn parenthesize_unary(&self, name: &str, expr: &ExprRef) -> String {
        format!("({} {})", name, expr.accept(self))
    }

    fn parenthesize_binary(&self, name: &str, expr1: &ExprRef, expr2: &ExprRef) -> String {
        format!("({} {} {})", name, expr1.accept(self), expr2.accept(self))
    }
}

impl<'a> VisitorRef<String> for AstPrinterRef<'a> {
    fn visit_expr(&self, expr: &ExprRef) -> String {
        match expr {
            &ExprRef::Binary {
                left,
                operator,
                right,
            } => self.parenthesize_binary(&operator.lexeme, left, right),
            &ExprRef::Grouping { expr } => self.parenthesize_unary("group", expr),
            &ExprRef::Literal { value } => match value {
                Some(value) => value.to_string(),
                None => "nil".to_owned(),
            },
            &ExprRef::Unary { operator, right } => self.parenthesize_unary(&operator.lexeme, right),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::scanner::{Literal, Token, TokenType};

    use super::*;

    #[test]
    fn test_ast_printer_ref() {
        // Expr expression = new Expr.Binary(
        //   new Expr.Unary(
        //     new Token(TokenType.MINUS, "-", null, 1),
        //     new Expr.Literal(123)),
        //   new Token(TokenType.STAR, "*", null, 1),
        //   new Expr.Grouping(
        //     new Expr.Literal(45.67)));

        let expr = ExprRef::Binary {
            left: &ExprRef::Unary {
                operator: &Token::new(TokenType::MINUS, "-".to_owned(), None, 1),
                right: &ExprRef::Literal {
                    value: Some(&Literal::Num(123.0)),
                },
            },
            operator: &Token::new(TokenType::STAR, "*".to_owned(), None, 1),
            right: &ExprRef::Grouping {
                expr: &ExprRef::Literal {
                    value: Some(&Literal::Num(45.67)),
                },
            },
        };

        let ast_printer = AstPrinterRef::new(&expr);

        assert_eq!(ast_printer.print(), "(* (- 123.0) (group 45.67))");
    }
}
