// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_vec_edge_009
pub fn run() -> Int {
    var v: Vec[Int16] = [];
    v.push(100i16);
    v.push(200i16);
    if v[0] == 100i16 && v[1] == 200i16 { return 0; }
    return 1;
  }
use m21_vec_edge_009.run;
fn main() -> Int { return run(); }
