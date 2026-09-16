// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_vec_edge_011
pub fn run() -> Int {
    var v: Vec[Int32] = [];
    v.push(1000i32);
    v.push(2000i32);
    if v[0] == 1000i32 && v[1] == 2000i32 { return 0; }
    return 1;
  }
use m21_vec_edge_011.run;
fn main() -> Int { return run(); }
