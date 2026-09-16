// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-G06: Constrained generic with Ord trait - Int only
fn larger[T: Ord](a: T, b: T) -> T { if a > b { return a; } return b; }
fn main() -> Int {
  if larger[Int](7, 3) != 7 { return 1; }
  if larger[Int](2, 9) != 9 { return 2; }
  if larger[Int](-5, 10) != 10 { return 3; }
  return 0;
}
