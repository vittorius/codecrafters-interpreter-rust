# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with
code in this repository. Max line length in this file must be 80 characters.

## What this is

A Rust implementation of an interpreter for Lox (from the book
[Crafting Interpreters](https://craftinginterpreters.com/)), built as a
CodeCrafters
["Build your own Interpreter"](https://app.codecrafters.io/courses/interpreter/overview)
challenge. Progress currently covers chapter 4 (Scanning) only — there is no
parser, AST, or evaluator yet, just a tokenizer.

## Commands

- Build: `cargo build`
- Run locally: `./your_program.sh tokenize <filename>` (builds a release binary
  into `/tmp/codecrafters-build-interpreter-rust` and runs it — this is the
  actual CodeCrafters compile/run flow reproduced locally, so prefer it over
  `cargo run` when checking end-to-end behavior)
- Run tests (default, complying with CodeCrafters test suite): `cargo test`
- Run tests (all, hidden behind features flags because of the CodeCrafters test
  suite): `cargo test --all-features`
- Submit to CodeCrafters: `codecrafters submit`

## Architecture

- `src/main.rs` — CLI entry point. Parses `argv`, currently only supports the
  `tokenize <filename>` command. Maps internal error state to CodeCrafters'
  expected process exit codes (`0` success, `64` usage error, `65` lexical
  error).
- `src/scanner.rs` — the tokenizer. Structured as two layers:
  - `Cursor<'a>`: a thin wrapper around a `Peekable<Chars<'a>>` that provides
    raw character-level lookahead (`advance`, `peek`, `peek_next`, `is_at_end`)
    with no knowledge of tokens or lexemes.
  - `Scanner<'a>`: owns a `Cursor` plus tokenizing state (`lexeme_cur` — the
    lexeme being accumulated, `line`, `tokens`, `has_error`). Its own
    `advance`/`peek`/etc. delegate to the `Cursor` but additionally accumulate
    consumed characters into `lexeme_cur`.
  - This split intentionally mirrors the structure `rustc_lexer` uses (cursor
    vs. tokenizer) — see the design goals in `ARCHITECTURE.md`.
- `src/lox.rs` — shared error reporting (`lox::error(line, message)`), printed
  in the `[line N] Error: message` format tests expect.
- `ARCHITECTURE.md` — states the scanner's three design goals: mirror the book's
  Java implementation's structure, use idiomatic Rust (iterators over
  `Chars<'a>` rather than byte indexing), and take inspiration from
  `rustc_lexer` without over-engineering. Keep new scanner code consistent with
  these goals.
- `tests/tokenize.rs` — integration tests for the scanner (currently a stub).

## Known incomplete areas

- Multiline `/* */` comments are partially implemented in
  `Scanner::comment_or_slash` (no nesting support yet, marked `TODO`).
- `KEYWORDS` uses a `HashMap` behind a `LazyLock`; there's a standing `TODO` to
  switch to the `phf` crate for a compile-time-built map.
- `Token::fmt` clones the `Literal` on every format call (marked `FIXME`).
