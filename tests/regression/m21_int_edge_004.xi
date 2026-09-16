// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_int_edge_004
pub fn run() -> Int {
    var a: Int16 = 32767i16;
    var b: Int16 = -32768i16;
    if a == 32767i16 && b == -32768i16 { return 0; }
    return 1;
  }
use m21_int_edge_004.run;
fn main() -> Int { return run(); }
