// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Generic identity with integer types
fn id[T](x: T) -> T { return x; }
fn main() -> Int {
  var a: Int8 = id[Int8](42);
  var b: Int16 = id[Int16](1000);
  var c: Int32 = id[Int32](100000);
  if a == 42 as Int8 && b == 1000 as Int16 && c == 100000 as Int32 {
    return 0;
  }
  return 1;
}
