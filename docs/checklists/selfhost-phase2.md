<!-- Copyright (c) 2026 Eleftherios Notas and The XIOM Authors -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# Selfhost Phase 2 -- Parser parity (checklist)

**Gate:** AST-dump equality for the corpus. `xiom --dump-ast` and
`xiomc-self --dump-ast` must produce the identical canonical dump
line-for-line over the Phase 0 corpus
(`cargo test -p xiom-codegen --test full_diff_tests`, `diff_ast`).

**Status: LANDED 2026-10-02.** Evidence below (gate green over the 83-file
corpus; bootstrap meter gate 2 flipped).

## Deliverables
- [x] Canonical `--dump-ast` on the Rust driver
      (`crates/xiom/src/main.rs::dump_ast`, `AstDump`), format owned there:
      `{indent}{Kind}[ key=value]... [span=l:c:bs:be]`; text payloads
      lowercase hex; Float literal payloads dump the SOURCE LEXEME (value
      parity deferred -- see below); `PARSE-ERROR` on a failed parse.
      `--dump-ast` added to the clap surface (`crates/xiom/src/cli.rs`).
- [x] Harness gate `full_diff_tests::diff_ast` (line-exact over the corpus
      manifest, same runner pattern as `diff_tokens`), green and
      un-ignored.
- [x] `--dump-ast` on the selfhost driver (`selfhost/src/main.xi`) routed to
      `selfhost_parser.dump_ast`.
- [x] AST model `selfhost/src/ast.xi`: flat `Vec[Node]` arena + Int child
      indices (`-1` = absent). Direct recursive VALUE enums are not usable
      (COMPILER_BUGS 2026-10-02 (e)); the arena resolution is invisible in
      the dump.
- [x] Port `crates/xiom-parser`:
      - `parser_state.xi` -- Parser state, token bridge, error latch,
        recovery, `tk_tag` kind identity (never `==`: COMPILER_BUGS (g)).
      - `parser_expr.xi` -- types, params, generics, derive lists, blocks,
        statements, patterns, all expression parsing (precedence ladder,
        postfix/generic calls, struct/array/tuple literals, closures, asm).
      - `parser_core.xi` -- program + file-module wrapping, top-level
        declarations, attributes, fn decls (receiver generics, where
        clauses, contracts), consts/module vars, externs.
      - `parser.xi` -- facade (lexer -> parser -> dump).
- [x] Mirror the dump walker in `selfhost/src/ast_dump.xi` byte-for-byte
      (same field order as `AstDump`).
- [x] Un-ignore `diff_ast`; update docs/SELFHOST_PROGRESS.md gate 2 row +
      Phase 2 evidence section.

## Port fidelity notes
- [x] Spans: POST-BOM `line:col:byte_start:byte_end`, identical to
      `--dump-tokens`; node spans mirror the Rust parser's choices
      (`Expr::Binary` takes the LEFT operand's span, array-size Int uses the
      post-advance peek span, `Ident::new` synthetic names reuse the
      declaration span, `Type::Named` node span = the name ident span).
- [x] Float literals: the AST dump emits the token lexeme (selfhost stores
      it in `NkLitFloat`), which equals the Rust dump's source slice at the
      node byte range for every float token the lexer produces.
- [x] Str literals: decoded bytes; the selfhost AST keeps literal payloads as
      `Vec[UInt8]` (XIOM `Str` is NUL-terminated).
- [x] Variant pattern names stay DOTTED as parsed (`Tree.Leaf`), matching the
      Rust `Pattern::Variant` name.
- [x] `expected` bitset / error text: diagnostic TEXT is not part of the
      dump gate; the port emits the single-token message and latches errors
      (Phase 3 owns diagnostic parity).
- [x] `XIOM_STRICT_BRACKETS`: LAX default mirrored (env opt-in not ported;
      not exercised by the corpus).

## Deferred (documented, not stubbed)
- [ ] Float VALUE parity in the AST (lexeme dump today; value parity lands
      with a correctly-rounded decimal->f64 parser / bitcast intrinsic).
- [ ] Trivia/comment attachment (the Rust parser has none; fmt/docgen only).
- [ ] Error-message/expected-set parity (Phase 3).

## Verification (2026-10-02)
- [x] Gate: `cargo test -p xiom-codegen --test full_diff_tests` ->
      `diff_ast` GREEN over the 83-file corpus (100.7 s), with T1
      `diff_corpus`, `diff_tokens` and `runtime_ffi_selfcheck` green in the
      same suite.
- [x] `cargo test -p xiom --bin xiom` -> 6 passed (clap surface incl.
      `--dump-ast`).
- [x] Rust-side smoke: `--dump-ast` rc 0 on the heavy corpus entries
      (benchmark_selfhost, phase1_hardening, m37_simd_runtime, ...).
- [x] `python tools/ascii_guard.py check` before every commit.
- [x] Findings filed in `docs/COMPILER_BUGS.md` 2026-10-02: (e) recursive
      enum payload boxing mis-lowers (crashes / pointer-as-value);
      (f) qualified variant patterns false W000; (g) `==` on enums with
      aggregate payloads emits `icmp %struct.Vec`; (h) `NkExprGenericCall`
      destructure field mis-mapping in a large dispatch function.
      Repros: `tmp/sprintc/phase2_parser/` (`probe_rec_*.xi`,
      `probe_nk_gc.xi`, `gc.xi`, agent repro dirs).
