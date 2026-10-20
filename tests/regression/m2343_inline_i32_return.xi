// m2343 (REL-20261015-2343-stdlib): a BARE inline i32 extern call read the
// full 64-bit register, so a negative return (-1) compared as >= 0.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// `let a = unsafe { extern_i32_fn(...) };` had no XIOM type/signedness record,
// so the comparison widened the i32 with `zext` (-1 -> 4294967295). The
// annotated inline (`let a: Int32 = unsafe { ... }`) and declared
// (`let a: Int32; unsafe { a = ... }`) forms were correct. The binding now
// keeps the UNSAFE BLOCK TAIL's return type (Int32), so both forms sign-extend.
//
// strcmp is deterministic: "a" vs "b" is negative; "b" vs "a" is positive.

module m2343_inline_i32_return

use xiom.io;

extern "C" {
  fn strcmp(a: *UInt8, b: *UInt8) -> Int32;
}

fn main() -> Int {
  var a: [2]UInt8;
  a[0] = 97 as UInt8;
  a[1] = 0 as UInt8;
  var b: [2]UInt8;
  b[0] = 98 as UInt8;
  b[1] = 0 as UInt8;

  // Bare inline form: must read the negative i32 as negative.
  let p = unsafe { strcmp(&a as *UInt8, &b as *UInt8) };
  if p >= 0 { return 1; }

  // Positive control: "b" vs "a" is > 0.
  let q = unsafe { strcmp(&b as *UInt8, &a as *UInt8) };
  if q <= 0 { return 2; }

  // Declared form stays correct.
  let d: Int32;
  unsafe { d = strcmp(&a as *UInt8, &b as *UInt8); }
  if d >= 0 { return 3; }

  io.println("i32_neg_ok");
  return 0;
}
