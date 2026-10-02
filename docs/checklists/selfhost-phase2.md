<!-- Copyright (c) 2026 Eleftherios Notas and The XIOM Authors -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# Selfhost Phase 2 -- Parser parity (checklist)

**Gate:** AST-dump equality for the corpus. `xiom --dump-ast` and
`xiomc-self --dump-ast` must produce the identical canonical dump
line-for-line over the Phase 0 corpus
(`cargo test -p xiom-codegen --test full_diff_tests -- --ignored diff_ast`).

**Status: IN PROGRESS (staged). The gate is NOT STARTED until the whole
corpus is green.** Staged sub-milestones land with the gate still honestly
NOT STARTED; docs/SELFHOST_PROGRESS.md gate 2 flips only at full-corpus
parity.

## Deliverables
- [x] Canonical `--dump-ast` on the Rust driver
      (`crates/xiom/src/main.rs::dump_ast`, `AstDump`), format owned there:
      `{indent}{Kind}[ key=value]... [span=l:c:bs:be]`; text payloads
      lowercase hex; Float literal payloads dump the SOURCE LEXEME (value
      parity deferred -- see below); `PARSE-ERROR` on a failed parse.
      `--dump-ast` added to the clap surface (`crates/xiom/src/cli.rs`).
- [x] Harness gate `full_diff_tests::diff_ast` (line-exact over the corpus
      manifest, same runner pattern as `diff_tokens`), `#[ignore]`d with the
      honest NOT STARTED reason until full parity; run explicitly with
      `-- --ignored diff_ast`.
- [x] `--dump-ast` on the selfhost driver (`selfhost/src/main.xi`), currently
      routed to the `selfhost_parser.dump_ast` stub (`PARSE-ERROR`).
- [ ] Port the AST model (`crates/xiom-ast`) to `selfhost/src/ast.xi`
      (recursive enums port directly: XIOM lowers recursive payloads as
      pointers -- probe `tmp/sprintc/phase2_parser/probe_rec.xi`).
- [ ] Port `crates/xiom-parser/src/lib.rs` to `selfhost/src/parser.xi` in
      stages, each keeping T1 (`diff_corpus`) and T2 green:
      1. statements/exprs (lexer bridge, Parser state, error/expected
         machinery, blocks, if/match/while/for, calls, literals)
      2. types
      3. patterns
      4. modules/imports/top-level declarations
      5. contracts
      6. generics
- [ ] Mirror the dump walker in `selfhost/src/ast_dump.xi` byte-for-byte
      (same field order as `AstDump`).
- [ ] Un-ignore `diff_ast` only when the whole corpus is line-exact; update
      docs/SELFHOST_PROGRESS.md gate 2 row + a Phase 2 evidence section.

## Port fidelity notes
- [ ] Spans: POST-BOM `line:col:byte_start:byte_end`, identical to
      `--dump-tokens`. Node spans must replicate the Rust parser's choices
      exactly (e.g. `Expr::Binary` takes the LEFT operand's span, type decls
      use the declaration keyword's span, `Ident` synthetic names reuse the
      declaration span).
- [ ] Float literals: the AST dump slices the SOURCE LEXEME at the node's
      byte range, because the selfhost has no correctly-rounded
      decimal->f64 parser (Phase 1 deferral). The selfhost parser does not
      need f64 value parity for this gate; it must carry the lexeme span
      exactly.
- [ ] Str literals: the dump emits the DECODED bytes. The selfhost AST keeps
      string-literal payloads as `Vec[UInt8]` (XIOM `Str` is NUL-terminated;
      `"\0"` would truncate; Phase 1 lexer parity has the same design).
- [ ] Variant pattern names are DOTTED as parsed (`Tree.Leaf`), matching the
      Rust `Pattern::Variant` name; the dump emits the dotted hex.
- [ ] `expected` bitset / error text machinery is NOT part of the dump gate
      (`PARSE-ERROR` has no payload); parsing error-message parity is a
      Phase 3 checker/diagnostics concern. The port still mirrors the
      control flow so valid-corpus parses stay identical.
- [ ] `XIOM_STRICT_BRACKETS`: the Rust default is LAX (`Vec<UInt8]`
      tolerated); the selfhost port mirrors the same default and reads the
      same env var if available.

## Deferred (documented, not stubbed)
- [ ] Float VALUE parity in the AST (lexeme dump today; value parity lands
      with a correctly-rounded decimal->f64 parser / bitcast intrinsic).
- [ ] Trivia/comment attachment (the Rust parser has none; fmt/docgen only).

## Verification
- [x] Rust `--dump-ast` smoke over corpus files (examples/phase1_full,
      stress_body_parser, diff_test, contracts, generics,
      m37_bug56_ensure_expr_body): rc 0, no panics.
- [x] `cargo test -p xiom-codegen --test full_diff_tests`: 3 passed
      (T1 `diff_corpus`, `diff_tokens`, `runtime_ffi_selfcheck`) + `diff_ast`
      ignored (honest NOT STARTED).
- [x] `cargo test -p xiom --bin xiom`: 6 passed (clap surface incl. the new
      `--dump-ast` flag).
- [ ] `python tools/ascii_guard.py check` before every commit.
- [ ] Findings filed in `docs/COMPILER_BUGS.md` as they are reproduced.
