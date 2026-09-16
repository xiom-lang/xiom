// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_vec_edge_002
pub fn run() -> Int {
    var v: Vec[Int] = [];
    var i = 0;
    while i < 1000 {
      v.push(i);
      i = i + 1;
    }
    if v.len() == 1000 && v[0] == 0 && v[999] == 999 { return 0; }
    return 1;
  }
use m21_vec_edge_002.run;
fn main() -> Int { return run(); }
