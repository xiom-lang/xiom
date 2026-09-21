// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-G10: Generic Vec type in function signatures
fn create_empty() -> Vec[Int] {
  return Vec[Int]{};
}
fn check_len(arr: Vec[Int]) -> Int {
  return 0;
}
fn main() -> Int {
  var v = create_empty();
  var n = check_len(v);
  if n != 0 { return 1; }
  return 0;
}
