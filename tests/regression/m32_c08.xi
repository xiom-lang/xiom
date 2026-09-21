// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-C08: Method contract -- impl method with requires/ensures
interface Check {
  fn between(self, lo: Int, hi: Int) -> Bool;
}
type Val = { n: Int; }
impl Check for Val {
  fn between(self, lo: Int, hi: Int) -> Bool
    requires: lo <= hi
    ensures: result == true => self.n >= lo
    ensures: result == true => self.n <= hi
  {
    return self.n >= lo && self.n <= hi;
  }
}
fn main() -> Int {
  var v = Val{ n: 7; };
  if v.between(0, 10) { return 0; }
  return 1;
}
