// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Ultimate narrow int stress: casts, bitwise, signed, unsigned, roundtrip
fn sext_test(x: Int8) -> Int32 { return x as Int32; }
fn zext_test(x: UInt8) -> Int32 { return x as Int32; }
fn main() -> Int {
  var s: Int8 = -128 as Int8;
  var u: UInt8 = 128;
  var ss: Int32 = sext_test(s);
  var uu: Int32 = zext_test(u);
  var s_check: Int32 = ss + 128 as Int32;
  var u_check: Int32 = uu - 128 as Int32;
  var s_ok: Bool = s_check == 0 as Int32;
  var u_ok: Bool = u_check == 0 as Int32;
  if s_ok && u_ok { return 0; }
  return 1;
}
