// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-J19: Module with method definition -- impl blocks and interfaces in modules
module impl_mod {
  pub type Counter = { value: Int; step: Int; }
  pub fn make(start: Int, step: Int) -> Counter {
    return Counter{ value: start; step: step; };
  }
}
type Counter = impl_mod.Counter;
interface Stepper { fn next(self) -> Counter; fn current(self) -> Int; }
impl Stepper for Counter {
  fn next(self) -> Counter {
    return Counter{ value: self.value + self.step, step: self.step };
  }
  fn current(self) -> Int { return self.value; }
}
use impl_mod.make;
fn main() -> Int {
  var c = make(10, 3);
  if c.current() != 10 { return 1; }
  var n = c.next();
  if n.current() != 13 { return 2; }
  return 0;
}
