// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-G15: Generic constraint with multiple method calls
interface Doubler { fn double(self) -> Int; }
interface Adder { fn add(self, x: Int) -> Int; }
type Val = { n: Int; }
impl Doubler for Val {
  fn double(self) -> Int { return self.n * 2; }
}
impl Adder for Val {
  fn add(self, x: Int) -> Int { return self.n + x; }
}
fn main() -> Int {
  var v = Val{ n: 10 };
  var d = v.double();
  var a = v.add(5);
  if d != 20 { return 1; }
  if a != 15 { return 2; }
  if d + a != 35 { return 3; }
  return 0;
}
