// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m37_f128
// BUG 13 regression: Float128 needs soft-float helpers (__divtf3 etc.)
// at link time and fp128 coercions on stores/params (not just As casts).

fn div128(a: Float128, b: Float128) -> Float128 {
  return a / b;
}

fn main() -> Int {
  var acc: Float128 = 1000;
  var i = 0;
  while i < 10 {
    acc = div128(acc, 2.5 as Float128);
    i = i + 1;
  }
  var x = acc as Float64;
  if x < 0.1 || x > 0.11 { return 1; }
  var n: Float128 = 5;
  var y = n as Float64;
  if y != 5.0 { return 2; }
  return 0;
}
