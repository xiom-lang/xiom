// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_vec_edge_022
pub fn run() -> Int {
    var v: Vec[Int] = [10, 20, 30, 40];
    v.remove(1);
    if v.len() == 3 && v[0] == 10 && v[1] == 30 && v[2] == 40 { return 0; }
    return 1;
  }
use m21_vec_edge_022.run;
fn main() -> Int { return run(); }
