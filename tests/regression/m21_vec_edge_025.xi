// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_vec_edge_025
pub fn run() -> Int {
    var v: Vec[Int] = [];
    var i = 0;
    while i < 5000 {
      v.push(i);
      i = i + 1;
    }
    if v.len() == 5000 && v[0] == 0 && v[4999] == 4999 { return 0; }
    return 1;
  }
use m21_vec_edge_025.run;
fn main() -> Int { return run(); }
