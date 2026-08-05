# Lox language tokenizer/parser/interpreter/compiler <https://craftinginterpreters.com/>

## Scanner design

This scanner is designed to achieve several following goals simultaneously:

1. Resemble the code structure of the Java implementation of the Lox scanner as
   much as possible for the ease of following the book.
2. Use idiomatic Rust constructs. (E.g. `Chars<'a>` iterator to traverse
   source's UTF-8 chars instead of treating the source as ASCII chars and index
   it byte-wise.)
3. Use `rustc_lexer` for inspiration but not over-engineer: we need to
   accomplish this project quickly.
