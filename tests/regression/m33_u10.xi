// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-U10: Unsafe write to pointer -- extern with pointer param, no call
extern "C" {
  fn write_int(dst: *Int, val: Int);
  fn write_float(dst: *UInt8, val: Float64);
}
fn main() -> Int {
  var x: Int = 0;
  var p: *Int;
  unsafe { p = &x as *Int; }
  var r: Int;
  unsafe { r = *p; }
  if r == 0 { return 0; }
  return 1;
}
