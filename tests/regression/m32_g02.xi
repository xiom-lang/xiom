// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-G02: Two-param generic function
fn pair[T](a: T, b: T) -> Int {
  return 0;
}
fn main() -> Int {
  var a: Int = pair(1, 2);
  var b: Int = pair(-5, -5);
  if a != 0 || b != 0 { return 1; }
  return 0;
}
