// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-L23: Unsafe block with raw pointer -- raw pointer cast and deref
fn main() -> Int {
  var x: Int = 65;
  var pi: *Int;
  var pu: *UInt8;
  unsafe {
    pi = &x as *Int;
    pu = pi as *UInt8;
  }
  var lo: UInt8;
  unsafe { lo = *pu; }
  if unsafe { *pi } == 65 { return 0; }
  return 1;
}
