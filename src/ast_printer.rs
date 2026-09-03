use crate::{
    environment::{BareEnv, Env, clone_env},
    expr::{Expr, VisitorMut},
};

pub struct AstPrinter<'a> {
    expr: &'a Expr<'a>,
}

impl<'a> AstPrinter<'a> {
    pub fn new(expr: &'a Expr<'_>) -> Self {
        Self { expr }
    }

    pub fn print(&mut self) -> String {
        self.visit_expr(self.expr, BareEnv::new().wrapped())
    }

    fn parenthesize_unary(&mut self, name: &str, expr: &'a Expr<'_>, env: Env) -> String {
        format!("({} {})", name, expr.accept(self, env))
    }

    fn parenthesize_binary(
        &mut self,
        name: &str,
        left: &'a Expr<'_>,
        right: &'a Expr<'_>,
        env: Env,
    ) -> String {
        format!(
            "({} {} {})",
            name,
            left.accept(self, clone_env(&env)),
            right.accept(self, env)
        )
    }

    fn parenthesize_ternary(
        &mut self,
        cond: &'a Expr<'_>,
        left: &'a Expr<'_>,
        right: &'a Expr<'_>,
        env: Env,
    ) -> String {
        format!(
            "(?: {} {} {})",
            cond.accept(self, clone_env(&env)),
            left.accept(self, clone_env(&env)),
            right.accept(self, env)
        )
    }

    fn parenthesize_assign(&mut self, name: &str, value: &'a Expr<'_>, env: Env) -> String {
        format!("(<- {} {})", name, value.accept(self, env))
    }
}

impl<'a> VisitorMut<'a, String> for AstPrinter<'a> {
    fn visit_expr(&mut self, expr: &'a Expr<'_>, env: Env) -> String {
        match expr {
            Expr::Binary {
                left,
                operator,
                right,
            } => self.parenthesize_binary(operator.lexeme, left, right, env),
            Expr::Conditional { cond, left, right } => {
                self.parenthesize_ternary(cond, left, right, env)
            }
            Expr::Grouping(expr) => self.parenthesize_unary("group", expr, env),
            Expr::Literal(value) => value.to_string(),
            Expr::Logical {
                left,
                operator,
                right,
            } => self.parenthesize_binary(operator.lexeme, left, right, env),
            Expr::Unary { operator, right } => self.parenthesize_unary(operator.lexeme, right, env),
            Expr::Variable(name) => name.lexeme.to_owned(),
            Expr::Assign { name, value } => self.parenthesize_assign(name.lexeme, value, env),
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
            left: Expr::Unary {
                operator: Token::new(TokenType::MINUS, "-", None, 1),
                right: Expr::Literal(Literal::Num(123.0)).boxed(),
            }
            .boxed(),
            operator: Token::new(TokenType::STAR, "*", None, 1),
            right: Expr::Grouping(Expr::Literal(Literal::Num(45.67)).boxed()).boxed(),
        };

        let mut ast_printer = AstPrinter::new(&expr);

        assert_eq!(ast_printer.print(), "(* (- 123.0) (group 45.67))");
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
        let mut ast_printer = AstPrinter::new(&expr);

        assert_eq!(ast_printer.print(), "(<- answer (+ 40.0 2.0))");
    }
}
