// m256 companion: the codegen-module slice -- Parser imported selectively,
// the scrutinee enum NOT imported (bare NkStmtMatch patterns resolve through
// the scrutinee), and the tuple-returning helper whose Vec payload binding
// hit the M19-erased i64 handle slot.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module m256_cgp

use m256_psp.Parser;

fn cgb_match_parts(p: &Parser, idx: Int) -> (Int, Vec[Int]) {
  match p.nodes[idx].kind {
    NkStmtMatch(scrut, arms) => { return (scrut, arms); },
    _ => { return (-1, Vec[Int].new()); },
  }
}

pub fn use_it(p: &Parser, idx: Int) -> Int {
  let parts = cgb_match_parts(p, idx);
  let scrut = parts.0;
  let arms = parts.1;
  return scrut + arms.len();
}
