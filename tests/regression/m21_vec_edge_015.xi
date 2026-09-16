// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_vec_edge_015
pub fn run() -> Int {
    var v: Vec[Int] = [10, 20, 30, 40, 50];
    if v[0] == 10 && v[2] == 30 && v[4] == 50 { return 0; }
    return 1;
  }
use m21_vec_edge_015.run;
fn main() -> Int { return run(); }
