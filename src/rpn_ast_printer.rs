use crate::{
    environment::{BareEnv, Env, clone_env},
    expr::{Expr, Visitor},
};

pub struct RpnAstPrinter<'a> {
    expr: &'a Expr,
}

impl<'a> RpnAstPrinter<'a> {
    pub fn new(expr: &'a Expr) -> Self {
        Self { expr }
    }

    pub fn print(&mut self) -> String {
        self.visit_expr(self.expr, BareEnv::new().wrapped())
    }

    fn format_unary(&self, name: &str, expr: &'a Expr, env: Env) -> String {
        format!("{} {}", expr.accept(self, env), name)
    }

    fn format_binary(
        &self,
        name: &str,
        left: &'a Expr,
        right: &'a Expr,
        env: Env,
    ) -> String {
        format!(
            "{} {} {}",
            left.accept(self, clone_env(&env)),
            right.accept(self, env),
            name
        )
    }

    fn format_call(&self, callee: &Expr, arguments: &[Expr], env: Env) -> String {
        let mut s = String::from("(");
        for arg in arguments {
            s.push_str(&format!("{} ", arg.accept(self, clone_env(&env))));
        }
        s.push_str(&format!("{})", callee.accept(self, clone_env(&env))));
        s
    }

    fn format_conditional(
        &self,
        cond: &'a Expr,
        left: &'a Expr,
        right: &'a Expr,
        env: Env,
    ) -> String {
        format!(
            "?: {} {} {}",
            cond.accept(self, clone_env(&env)),
            left.accept(self, clone_env(&env)),
            right.accept(self, env),
        )
    }

    fn format_assign(&self, name: &str, value: &'a Expr, env: Env) -> String {
        format!("{} {} <-", name, value.accept(self, env))
    }
}

impl Visitor<String> for RpnAstPrinter<'_> {
    fn visit_expr(&self, expr: &Expr, env: Env) -> String {
        match expr {
            Expr::Binary {
                left,
                operator,
                right,
            } => self.format_binary(&operator.lexeme, left, right, env),
            Expr::Call {
                callee,
                paren,
                arguments,
            } => self.format_call(callee, arguments, env),
            Expr::Conditional { cond, left, right } => {
                self.format_conditional(cond, left, right, env)
            }
            Expr::Grouping(expr) => expr.accept(self, env),
            Expr::Literal(value) => value.to_string(),
            Expr::Logical {
                left,
                operator,
                right,
            } => self.format_binary(&operator.lexeme, left, right, env),
            Expr::Unary { operator, right } => self.format_unary(&operator.lexeme, right, env),
            Expr::Variable(name) => name.lexeme.to_owned(),
            Expr::Assign { name, value } => self.format_assign(&name.lexeme, value, env),
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
        let mut ast_printer = RpnAstPrinter::new(&expr);

        assert_eq!(ast_printer.print(), "1.0 2.0 + 4.0 3.0 - *");
    }

    #[test]
    fn test_assignment_expression() {
        let expr = Expr::Assign {
            name: Token::new(TokenType::IDENTIFIER, "answer".to_owned(), None, 1),
            value: Expr::Binary {
                left: Expr::Literal(Literal::Num(40.0)).boxed(),
                operator: Token::new(TokenType::PLUS, "+".to_owned(), None, 1),
                right: Expr::Literal(Literal::Num(2.0)).boxed(),
            }
            .boxed(),
        };
        let mut ast_printer = RpnAstPrinter::new(&expr);

        assert_eq!(ast_printer.print(), "answer 40.0 2.0 + <-");
    }
}
