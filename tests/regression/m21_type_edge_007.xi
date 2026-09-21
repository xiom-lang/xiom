// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_type_edge_007
pub fn run() -> Int {
    var v: Vec[Vec[Int]] = [[1, 2], [3, 4, 5]];
    if v[0][0] == 1 && v[1][2] == 5 { return 0; }
    return 1;
  }
use m21_type_edge_007.run;
fn main() -> Int { return run(); }
