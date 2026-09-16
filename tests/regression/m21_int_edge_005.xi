// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_int_edge_005
pub fn run() -> Int {
    var a: Int32 = 2147483647i32;
    var b: Int32 = -2147483648i32;
    if a == 2147483647i32 && b == -2147483648i32 { return 0; }
    return 1;
  }
use m21_int_edge_005.run;
fn main() -> Int { return run(); }
