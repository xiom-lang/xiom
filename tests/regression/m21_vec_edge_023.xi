// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_vec_edge_023
pub fn run() -> Int {
    var v: Vec[Int] = [1, 2, 3];
    v.clear();
    if v.len() == 0 { return 0; }
    return 1;
  }
use m21_vec_edge_023.run;
fn main() -> Int { return run(); }
