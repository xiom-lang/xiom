// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-U08: Raw pointer cast -- cast between pointer types
fn main() -> Int {
  var x: Int = 65;
  var pi: *Int;
  unsafe { pi = &x as *Int; }
  var pu: *UInt8;
  unsafe { pu = pi as *UInt8; }
  if unsafe { *pi } == 65 { return 0; }
  return 1;
}
