// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_match_edge_009
enum Bin { Zero, One }

  pub fn run() -> Int {
    var b = Bin.One;
    var result = match b {
      Bin.Zero => 0,
      Bin.One => 42,
    };
    if result == 42 { return 0; }
    return 1;
  }
use m21_match_edge_009.run;
fn main() -> Int { return run(); }
