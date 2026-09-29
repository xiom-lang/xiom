// XIOM -- Selfhost lexer (Phase 0 skeleton)
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Phase 1 ports crates/xiom-lexer/src/lib.rs to this module 1:1 (token
// kinds, spans, the block-comment and `\xNN`-codepoint rules) and adds a
// `--dump-tokens` parity mode gated on byte-equal dumps.

module selfhost_lexer

/// Phase 0 stub: number of tokens in `src`. Returns 0 until Phase 1.
pub fn lex_count(src: Str) -> Int {
  return 0;
}
