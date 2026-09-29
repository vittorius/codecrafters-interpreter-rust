# AGENTS.md

This file provides guidance to AI coding agents working with code in this
repository. Max line length in this file must be 80 characters (markdown prose
is wrapped by prettier: `printWidth: 80`, `proseWrap: always`). `CLAUDE.md` is a
symlink to this file — edit only `AGENTS.md`.

## What this is

A Rust implementation of an interpreter for Lox (from the book
[Crafting Interpreters](https://craftinginterpreters.com/)), built as a
CodeCrafters
["Build your own Interpreter"](https://app.codecrafters.io/courses/interpreter/overview)
challenge. Complete pipeline: scanner, recursive-descent parser, tree-walking
interpreter with a resolver, and a rustyline-based REPL. Extensions beyond the
book are gated behind cargo feature flags (see below). Binary-only crate (no
`lib.rs`); edition 2024, `rust-version = 1.96` (matches the CodeCrafters
`rust-1.96` buildpack in `codecrafters.yml`).

## Commands

- Build: `cargo build`
- Run locally: `./your_program.sh (tokenize|parse|evaluate|run) <filename>`
  (builds a release binary into `/tmp/codecrafters-build-interpreter-rust` and
  runs it — this reproduces the CodeCrafters compile/run flow from
  `.codecrafters/compile.sh` + `run.sh`, so prefer it over `cargo run` when
  checking end-to-end behavior)
- REPL: `cargo run -- repl` (rustyline-based; loads `history.txt` from the
  current dir if present and saves it on exit; internal commands `:help` and
  `:quit`; Ctrl+C/Ctrl+D exit gracefully like `:quit`, with code `0`)
- REPL multiline input (evcxr-style): Enter continues the line while the input
  is lexically incomplete (unclosed bracket/string/block comment, trailing
  operator or `else`) instead of submitting; Enter on an empty continuation line
  (buffer ends with `\n\n`) force-submits so the parser can report the error;
  Ctrl+J and Alt+Enter insert a newline unconditionally. Multiline snippets
  round-trip through `history.txt` (the `#V2` format escapes newlines) and
  Up/Down navigate them (`LineUpOrPreviousHistory`)
- Run tests (default features only, complying with the CodeCrafters test suite):
  `cargo test`
- Run all tests including feature-gated ones: `cargo test --all-features`
- Single test suite: `cargo test --test run --all-features`; single test: append
  a name filter, e.g. `cargo test --test run lambda --all-features`
- Lint: `cargo clippy --all-features` (config lives in `[lints]` in
  `Cargo.toml`: `clippy::unwrap_used`, `correctness`, and `style` are denied;
  `pedantic` and others warn — use `.expect("...")` over `.unwrap()`)
- Format: `cargo fmt`, but `rustfmt.toml` sets the nightly-only `group_imports`;
  stable fmt warns and skips it. Use `cargo +nightly fmt` to fully comply.
- Verify every successful build/change with BOTH `cargo test --all-features` and
  `codecrafters test -previous` (server-side rerun of all previously passed
  challenge stages, without committing) — this is a CodeCrafters challenge, so
  never break an already-passed stage
- Submit to CodeCrafters: `codecrafters submit`

## Feature flags

Features in `Cargo.toml` (`conditional-op`, `init-vars`, `str-cmp`,
`str-num-concat`, `comma-op`, `lambdas`, `rpn-ast-printer`, `err-unused-vars`,
`readline`) gate both implementation code (`#[cfg(feature = ...)]` scattered
through `parser.rs`, `interpreter.rs`, `expr.rs`, `printer/`) and the tests
exercising them. A plain `cargo build`/`test`/`clippy` silently skips all of it
— verify with `--all-features` when touching those files.

The `readline` feature adds a `readline()` native function (a
`NativeFunction` in `Interpreter::new()`), which reads one line from the OS
stdin minus its trailing `\r\n`. It works in `run` mode and inside the REPL:
rustyline restores the terminal to canonical mode once a submitted line is
returned, so a plain `stdin().read_line` blocks on the user's next Enter with
the tty echoing and editing the input. The `? ` prompt is printed only when
stdin is a terminal, so piped input (`tests/repl.rs` drives the whole REPL
this way) never sees it. REPL integration tests live in `tests/repl.rs` and
script-mode ones in the `readline_tests` module of `tests/run.rs`.

## Architecture

- `src/main.rs` — CLI entry point. `tokenize|parse|evaluate|run <filename>` plus
  `repl`. Maps errors to exit codes that tests assert on: `0` success, `64`
  usage, `65` lexical/syntactic/resolution error, `70` runtime error.
  `parse`/`evaluate` use `Parser::parse_expr` (a single expression); `run` uses
  `Parser::parse` (statements) then `Resolver` then `Interpreter`.
- `src/repl.rs` — rustyline REPL with a custom `Highlighter`: reuses `Scanner`
  to colorize each input line with a Gruvbox Dark truecolor palette (tokens via
  `Token.start` byte offsets, comments detected in between-token gaps; on scan
  errors the valid prefix is highlighted from `ScanError`'s partial tokens). Its
  `Validator` drives multiline input: `is_incomplete_fragment` is a small
  lexical fragment checker (idea borrowed from evcxr) that mirrors the `Scanner`
  semantics (nested block comments, escape-less multiline strings) and is
  deliberately conservative: it submits anything it isn't sure is unfinished so
  the parser reports the error instead of trapping the user
- `src/scanner.rs` — the tokenizer, structured as two layers:
  - `Cursor<'a>`: a thin wrapper around a `Peekable<Chars<'a>>` providing raw
    character-level lookahead (`advance`, `peek`, `peek_next`, `is_at_end`) with
    no knowledge of tokens or lexemes.
  - `Scanner<'a>`: owns a `Cursor` plus tokenizing state (`lexeme_cur`, `line`,
    `tokens`, `has_error`); its `advance`/`peek`/etc. delegate to the `Cursor`
    but accumulate consumed characters into `lexeme_cur`.
  - This split intentionally mirrors `rustc_lexer` (cursor vs. tokenizer) — see
    the design goals in `ARCHITECTURE.md` and keep new scanner code consistent
    with them.
  - The custom `scanner::Result`'s `Err(ScanError(ErrorSink, Tokens))` still
    carries the partially scanned tokens.
  - Tokens carry their `start` byte offset in the source (used by the REPL
    highlighter); `Token`'s `Display` format is asserted by the CodeCrafters
    tokenize stage — don't change it.
- `src/interpreter/` — tree-walking interpreter: `value`, `environment`,
  callables (`function`, `native_function`, `class`, `instance`), `resolver`
  (with `scope.rs`), and `expr_visitor`/`stmt_visitor`.
- `src/printer/` — `AstPrinter` (Lisp-style, used by the `parse` command) and
  the feature-gated `rpn_ast_printer`.
- `src/error.rs` — `ErrorSink` (collects errors so scanning/parsing can
  continue) and `RuntimeError`. `src/lox.rs` — error string formatting; tests
  match the `[line N] Error: message` format it produces.
- `tests/` — end-to-end integration tests: they run the compiled binary via
  `CARGO_BIN_EXE_codecrafters-interpreter` against temp `.lox` files (helpers in
  `tests/common/mod.rs`), asserting on stdout/stderr/exit codes.
