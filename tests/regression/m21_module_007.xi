// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_module_007
pub type Inner = { val: Int; }

  pub fn make(v: Int) -> Inner {
    return { val: v; };
  }

  pub fn run() -> Int {
    var x = make(99);
    if x.val == 99 { return 0; }
    return 1;
  }
use m21_module_007.run;
fn main() -> Int { return run(); }
