// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_vec_edge_001
pub fn run() -> Int {
    var v: Vec[Int] = [];
    v.push(42);
    if v.len() == 1 { return 0; }
    return 1;
  }
use m21_vec_edge_001.run;
fn main() -> Int { return run(); }
