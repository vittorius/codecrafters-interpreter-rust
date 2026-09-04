// TODO: implement user-defined functions


// use std::fmt::Display;

// use crate::callable::Callable;

// struct FunctionDeclaration {
//     name: String,
//     params: Vec<String>,
//     // body: Vec<Stmt<'a>>,
// }

// #[derive(Debug)]
// pub struct Function<'a> {
//     declaration: FunctionDeclaration,
// }

// impl Callable for Function<'_> {
//     fn arity(&self) -> usize {
//         todo!()
//     }

//     fn call(
//         &self,
//         interpreter: &crate::interpreter::Interpreter,
//         arguments: &[crate::value::Value],
//         env: crate::environment::Env,
//     ) -> crate::value::Value {
//         todo!()
//     }
// }

// impl Display for Function<'_> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "<fn {}>", self.declaration.name.lexeme)
//     }
// }
