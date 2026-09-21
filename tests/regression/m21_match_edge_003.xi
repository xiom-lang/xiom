// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_match_edge_003
enum Op { Add, Sub, Mul, Div }

  pub fn run() -> Int {
    var op = Op.Mul;
    match op {
      Op.Add => return 1,
      Op.Sub => return 1,
      Op.Mul => return 0,
      Op.Div => return 1,
    }
  }
use m21_match_edge_003.run;
fn main() -> Int { return run(); }
