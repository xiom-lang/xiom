// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_match_edge_010
pub fn run() -> Int {
    var n = 7;
    match n {
      0 | 1 | 2 => return 1,
      3 | 4 | 5 => return 1,
      6 | 7 | 8 => return 0,
      _ => return 1,
    }
  }
use m21_match_edge_010.run;
fn main() -> Int { return run(); }
