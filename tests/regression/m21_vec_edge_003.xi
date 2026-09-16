// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_vec_edge_003
pub fn run() -> Int {
    var v: Vec[Int] = [];
    v.push(1);
    v.push(2);
    v.push(3);
    v.push(4);
    v.push(5);
    if v.len() == 5 { return 0; }
    return 1;
  }
use m21_vec_edge_003.run;
fn main() -> Int { return run(); }
