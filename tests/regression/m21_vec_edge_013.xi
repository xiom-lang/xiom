// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_vec_edge_013
pub fn run() -> Int {
    var v: Vec[Option[Int]] = [];
    v.push(Some(1));
    v.push(None);
    v.push(Some(3));
    if v.len() == 3 { return 0; }
    return 1;
  }
use m21_vec_edge_013.run;
fn main() -> Int { return run(); }
