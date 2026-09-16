// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_int_edge_006
pub fn run() -> Int {
    var a: Int = 0;
    var b: Int = -1;
    var c: Int = 1;
    if a == 0 && b == -1 && c == 1 { return 0; }
    return 1;
  }
use m21_int_edge_006.run;
fn main() -> Int { return run(); }
