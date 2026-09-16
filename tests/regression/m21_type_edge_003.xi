// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_type_edge_003
pub fn run() -> Int {
    var a: Int64 = 10000000000i64;
    if a > 0 { return 0; }
    return 1;
  }
use m21_type_edge_003.run;
fn main() -> Int { return run(); }
