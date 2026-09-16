// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Generic add function with integer types
fn add[T](a: T, b: T) -> T { return a + b; }
fn main() -> Int {
  var a: Int32 = add[Int32](100, 200);
  var b: Int16 = add[Int16](10, 20);
  if a == 300 as Int32 && b == 30 as Int16 {
    return 0;
  }
  return 1;
}
