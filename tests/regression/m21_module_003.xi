// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_module_003
pub fn double(x: Int) -> Int { return x * 2; }
  pub fn triple(x: Int) -> Int { return x * 3; }

  pub fn run() -> Int {
    var a = double(5);
    var b = triple(5);
    if a == 10 && b == 15 { return 0; }
    return 1;
  }
use m21_module_003.run;
fn main() -> Int { return run(); }
