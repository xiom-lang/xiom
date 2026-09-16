// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-N2-10: Function factory pattern -- function creates and returns closure
fn factory(n: Int) -> Int {
  var f = |x| n + x;
  return f(50);
}
fn main() -> Int {
  var r1 = factory(10);
  var r2 = factory(100);
  if r1 == 60 && r2 == 150 { return 0; }
  return 1;
}
