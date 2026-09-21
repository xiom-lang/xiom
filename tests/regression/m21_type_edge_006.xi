// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_type_edge_006
type CharHolder = { c: Char; }

  pub fn run() -> Int {
    var h: CharHolder = { c: 'A'; };
    if h.c == 'A' { return 0; }
    return 1;
  }
use m21_type_edge_006.run;
fn main() -> Int { return run(); }
