// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Function param Int16
fn sub16(a: Int16, b: Int16) -> Int16 {
  return a - b;
}
fn main() -> Int {
  var c: Int16 = sub16(-32768 as Int16, 1);
  if c == 32767 { return 0; }
  return 1;
}
