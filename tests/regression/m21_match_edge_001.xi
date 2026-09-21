// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_match_edge_001
pub fn run() -> Int {
    var x = 1;
    match x {
      1 | 2 | 3 => return 0,
      4 | 5 | 6 => return 1,
      _ => return 2,
    }
  }
use m21_match_edge_001.run;
fn main() -> Int { return run(); }
