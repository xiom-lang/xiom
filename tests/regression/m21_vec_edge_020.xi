// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_vec_edge_020
pub fn run() -> Int {
    var v: Vec[Int] = [5, 2, 8, 1, 9, 3];
    v.sort();
    if v[0] == 1 && v[5] == 9 { return 0; }
    return 1;
  }
use m21_vec_edge_020.run;
fn main() -> Int { return run(); }
