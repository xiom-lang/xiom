// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-M15: Complex module -- fn, type, const, nested module all exported
module core {
  pub const VERSION: Int = 1;
  pub type Id = Int;
  pub fn make_id(n: Int) -> Id { return n; }
  pub fn check_version(v: Int) -> Bool { return v == VERSION; }
  module utils {
    pub fn double(x: Int) -> Int { return x * 2; }
    pub fn triple(x: Int) -> Int { return x * 3; }
  }
}
use core.make_id;
use core.check_version;
use core.utils.double;
use core.utils.triple;
fn main() -> Int {
  var id = make_id(42);
  var d = double(5);
  var t = triple(5);
  if id == 42 && check_version(1) && d == 10 && t == 15 { return 0; }
  return 1;
}
