// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Pointer to int cast through Int64 with narrow intermediate
fn main() -> Int {
  var x: Int = 42;
  var p: &Int = &x;
  var addr: Int = p as Int;
  if addr == addr { return 0; }
  return 1;
}
