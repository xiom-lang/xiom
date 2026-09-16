// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_match_edge_002
pub fn run() -> Int {
    var x = 50;
    match x {
      v if v > 100 => return 1,
      v if v > 25 => return 0,
      _ => return 1,
    }
  }
use m21_match_edge_002.run;
fn main() -> Int { return run(); }
