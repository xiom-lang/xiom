<!-- Copyright (c) 2026 Eleftherios Notas and The XIOM Authors -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# Selfhost Phase 1 -- Lexer parity (checklist)

**Gate:** token-dump equality for the corpus. `xiom --dump-tokens` and
`xiomc-self --dump-tokens` must produce the identical canonical dump
line-for-line over the Phase 0 corpus.

**Status: LANDED 2026-10-02.** Evidence below.

## Deliverables
- [x] Port `crates/xiom-lexer/src/lib.rs` to `selfhost/src/lexer.xi`:
      TokenKind/Token model, char-indexed scanning, BOM stripping, Unicode
      whitespace, comments/shebang, all literal kinds, the exact error text.
- [x] `--dump-tokens` on BOTH compilers, one byte-stable format:
      `crates/xiom/src/main.rs::dump_tokens` owns the definition
      (`{line}:{col}:{byte_start}:{byte_end} {TAG}[ {PAYLOAD}]`, lowercase
      hex payloads); `selfhost/src/lexer.xi::dump_tokens` mirrors it.
      `--dump-tokens` added to the clap surface (`crates/xiom/src/cli.rs`).
- [x] Harness gate: `crates/xiom-codegen/tests/full_diff_tests.rs::diff_tokens`
      runs both dumps over the corpus and asserts equality (`lines_of`
      normalizes the CRT CRLF the XIOM side emits on Windows pipes).

## Port fidelity notes
- [x] Spans: line/col 1-based; `byte_start`/`byte_end` are POST-BOM byte
      offsets. `byte` is an INDEPENDENT accumulator (Rust parity): an
      invalid numeric suffix restores `pos` only, so `byte`/`col` stay
      advanced and token text is built char-wise, never by slicing source
      bytes (`42i9` reproduces the shifted-span quirk exactly).
- [x] `\xNN` = codepoint U+00NN; `\u{...}` accepts hex digits and falls back
      to U+FFFD for empty/overflowing/surrogate codepoints, then U+FFFD for
      surrogates/out-of-range (Rust `char::from_u32` semantics).
- [x] Big integers: u128 range checked textually (leading zeros stripped;
      >39 decimal digits or a >u128::MAX 39-digit value / >32 hex digits is
      an Error token with the exact Rust message); values > u64::MAX become
      `TkBigInt(hi, lo)`.
- [x] Float token payload uses `string.str_to_float`; MALFORMED shapes
      (underscores in the int part, exponent without digits) return the
      exact `malformed float literal ...` Error token without consuming the
      suffix (Rust returns before `parse_numeric_suffix`).
- [x] Str payload is the decoded BYTE VECTOR: XIOM `Str` is NUL-terminated,
      so `"\0"` would truncate a `Str` payload (Rust preserves the NUL).
      The parser only reads lexemes for Int/BigInt/Float suffixes, so the
      Str lexeme difference is inert.
- [x] TokenKind variants are `Tk`-prefixed (bare `Some`/`None`/`True`/
      `False` collide with builtin variants/literals; `Pipe` with
      `xiom.os.Pipe`); `lx_kind_tag` maps them back to the Rust names.
- [x] Lexer internals are free `&mut Lexer` functions (`lx_` prefix), not
      receiver methods: nested `self` method mutation does not propagate in
      the current compiler (COMPILER_BUGS 2026-10-02 (a);
      `tmp/sprintc/phase1_lexer/probe8.xi` vs `probe9.xi`).

## Deferred (documented, not stubbed)
- [ ] Float VALUE parity: the dump uses the token LEXEME for Float tokens.
      The selfhost side has no correctly-rounded decimal->f64 parser (the
      stdlib's accumulator parser is not correctly rounded) and no
      i64<->f64 bitcast intrinsic (`num.float::float_bits` is a documented
      0-returning fallback). Value parity will surface in the parser/AST
      phase dumps; the orthography, classification, span, and lexeme are
      already gated.
- [ ] Trivia (comments/shebang) collection: Phase 1 dumps tokens only; the
      Rust `tokenize_with_trivia` surface lands if/when fmt/docgen is
      ported.

## Verification (2026-10-02)
- [x] Gate: `cargo test -p xiom-codegen --test full_diff_tests` -> **3
      passed** (`diff_tokens` over the 83-file corpus, `diff_corpus` T1
      49.2 s-class, `runtime_ffi_selfcheck`), 72.7 s total.
- [x] `cargo test -p xiom --bin xiom` -> 6 passed (clap surface incl. the
      new `--dump-tokens` flag).
- [x] Torture parity (line-exact): BOM / double-BOM, CRLF, empty file,
      shebang, unicode whitespace, NBSP, emoji + CJK + `\u{1F600}` strings,
      embedded NUL (`"\0"`), escapes, hex/decimal overflows, `0x`, `1e`,
      `1_000.5`, `42i9`, `.5`, lone `@`, unterminated string/char/block
      comment. Fixtures: `tmp/sprintc/phase1_lexer/torture*.xi`.
- [x] `python tools/ascii_guard.py check` -> clean.
- [x] Findings filed: `docs/COMPILER_BUGS.md` 2026-10-02 (nested method
      mutation; 128-bit enum payload; inf float literal; unsigned
      formatting/division).
