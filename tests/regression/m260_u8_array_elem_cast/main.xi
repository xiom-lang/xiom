// m260 (C-ORBIT-08 B): [N]UInt8 element reads must ZERO-extend through
// `as Int` / implicit widening (0xF8 -> 248; they sign-extended to -8
// before because annotation-only array locals recorded no element type).
// [N]Int8 elements must keep sign extension.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module m260_u8_array_elem_cast;

fn u8_at_one_point_five() -> Int
  requires: true
{
  unsafe {
    var b: [8]UInt8;
    let fp: *mut Float64 = &b[0] as *mut Float64;
    *fp = 1.5;
    return b[6] as Int;
  }
}

fn i8_elem() -> Int {
  var v: Int8 = 0 - 8;
  let b: [1]Int8 = [v];
  return b[0] as Int;
}

fn main() -> Int {
  if u8_at_one_point_five() != 248 { return 1; }
  if i8_elem() != -8 { return 2; }
  return 0;
}
