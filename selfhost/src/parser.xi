// XIOM -- Selfhost parser (Phase 0 skeleton)
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Phase 2 ports crates/xiom-parser to this module (statements/exprs ->
// types -> patterns -> modules/imports -> contracts -> generics) and adds a
// `--dump-ast` parity mode gated on byte-equal dumps.

module selfhost_parser

/// Phase 0 stub: number of AST nodes parsed from `src`. Returns 0 until
/// Phase 2.
pub fn parse_count(src: Str) -> Int {
  return 0;
}
