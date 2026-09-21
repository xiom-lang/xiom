// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_module_004
pub type Id = Int;
  pub type Name = Str;

  pub fn run() -> Int {
    var id: Id = 42;
    var name: Name = "test";
    if id == 42 { return 0; }
    return 1;
  }
use m21_module_004.run;
fn main() -> Int { return run(); }
