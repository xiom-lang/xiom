// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_module_002
pub type Pair = { a: Int; b: Int; }

  pub fn run() -> Int {
    var p: Pair = { a: 1; b: 2; };
    if p.a == 1 && p.b == 2 { return 0; }
    return 1;
  }
use m21_module_002.run;
fn main() -> Int { return run(); }
