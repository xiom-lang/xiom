// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-H12: Int->Bool pattern -- non-zero is truthy, zero is falsy
fn int_to_bool(i: Int) -> Bool { if i != 0 { return true; } return false; }
fn main() -> Int {
  if int_to_bool(42) == true && int_to_bool(0) == false { return 0; }
  return 1;
}
