// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_int_edge_001
pub fn run() -> Int {
    var a: Int8 = 127i8;
    if a == 127i8 { return 0; }
    return 1;
  }
use m21_int_edge_001.run;
fn main() -> Int { return run(); }
