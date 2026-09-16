// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_vec_edge_008
pub fn run() -> Int {
    var v: Vec[Int] = [5, 10, 15];
    if v.len() == 3 { return 0; }
    return 1;
  }
use m21_vec_edge_008.run;
fn main() -> Int { return run(); }
