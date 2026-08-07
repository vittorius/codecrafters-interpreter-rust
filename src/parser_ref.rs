// use crate::{
//     expr::{Expr, ExprRef},
//     scanner::{Token, TokenType, TokenType as TT},
// };

// pub struct ParserRef<'a> {
//     tokens: &'a [Token],
//     current: usize,
//     pub has_error: bool,
// }

// impl<'a> ParserRef<'a> {
//     pub fn new(tokens: &'a [Token]) -> Self {
//         ParserRef {
//             tokens,
//             current: 0,
//             has_error: false,
//         }
//     }

//     pub fn parse(&self) -> &ExprRef {
//         todo!()
//     }

//     fn matches(token_types: &[TokenType]) -> bool {
//         for tt in token_types {
//             if check(tt) {
//                 advance();
//                 return true;
//             }
//         }

//         false
//     }

//     fn previous(&self) -> &Token {
//         &self.tokens[self.current - 1]
//     }

//     fn expression(&self) -> Expr {
//         equality(self)
//     }

//     fn equality(&self) -> Expr {
//         let expr = comparison(self);

//         while Self::matches(&[TT::BANG_EQUAL, TT::EQUAL_EQUAL]) {
//             let operator = self.previous();
//         }
//     }

//     fn comparison(&self) -> Expr {
//         todo!()
//     }
// }

// enum Exp<'a> {
//     Binary {
//         left: &'a Exp<'a>,
//         op: &'a Tok,
//         right: &'a Exp<'a>,
//     },
//     Unary {
//         op: &'a Tok,
//         right: &'a Exp<'a>,
//     },
//     Literal(Lit)
// }

// enum Lit {
//     Num(f64),
//     Str(String)
// }

// enum TokType {
//     LIT,
//     NEG,
//     PLUS,
//     MINUS
// }

// struct Tok {
//    tt: TokType,
//   lexeme: String,
// }

// struct Holder<'a> {
//     toks: Vec<Tok>,
//     current: usize,
//     exps: Vec<&'a Exp<'a>>,
// }

// impl<'a> Holder<'a> {
//     fn new(toks: Vec<Tok>) ->Self {
//        Self {
//            toks,
//            current: 0,
//            exps: vec![]
//        }
//     }
    
//     fn produce() -> Exp<'a> {

//     }

//     fn produce_binary() {

//     }

//     fn produce_unary(&self) -> Exp<'a> {
//        Exp::Unary { op: self.toks[self.current], right: &Exp::Literal(Lit::Str("")) }
//     }
// }
