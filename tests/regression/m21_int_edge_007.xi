// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_int_edge_007
pub fn run() -> Int {
    var a: Int = 42;
    var b: Int = a % 10;
    var c: Int = a / 10;
    var d: Int = a * a;
    if b == 2 && c == 4 && d == 1764 { return 0; }
    return 1;
  }
use m21_int_edge_007.run;
fn main() -> Int { return run(); }
