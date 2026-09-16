// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_vec_edge_024
pub fn run() -> Int {
    var v: Vec[Int] = [10, 20, 30, 40, 50];
    var sum = 0;
    var i = 0;
    while i < v.len() {
      sum = sum + v[i];
      i = i + 1;
    }
    if sum == 150 { return 0; }
    return 1;
  }
use m21_vec_edge_024.run;
fn main() -> Int { return run(); }
