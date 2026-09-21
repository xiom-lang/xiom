// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-H20: Cast in generic context -- generic identity then cast to Int
fn id[T](x: T) -> T { return x; }
fn main() -> Int {
  var a: Int8 = 77;
  var b: Int8 = id(a);
  var r: Int = b as Int;
  if r == 77 { return 0; }
  return 1;
}
