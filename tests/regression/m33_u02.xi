// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-U02: Extern function declarations -- multiple fn signatures, no calls
extern "C" {
  fn dummy_abs(x: Int) -> Int;
  fn dummy_min(a: Int, b: Int) -> Int;
}
fn main() -> Int {
  var x: Int = 0;
  unsafe { x = 42; }
  if x == 42 { return 0; }
  return 1;
}
