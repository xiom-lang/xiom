// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module types_mod {
  pub type Color = {
    r: Int;
    g: Int;
    b: Int;
  }
}

module user_mod {
  fn test() -> Int {
    var c = Color{ r: 255, g: 128, b: 64 };
    if c.r == 255 { return 1; }
    return 0;
  }

  pub fn run_test() -> Int { return test(); }
}

use user_mod.run_test;

fn main() -> Int {
  return run_test();
}
