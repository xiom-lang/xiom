// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_type_edge_002
pub fn run() -> Int {
    var a: Float32 = 3.14;
    var b: Float64 = 2.718;
    if a == 3.14 { return 0; }
    return 1;
  }
use m21_type_edge_002.run;
fn main() -> Int { return run(); }
