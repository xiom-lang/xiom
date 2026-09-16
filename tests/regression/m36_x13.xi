// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-X13: Overflow checks -- integer overflow/underflow boundary behavior
fn main() -> Int {
  var max_int: Int = 2147483647;
  var min_int: Int = -2147483647 - 1;
  if max_int + (-1) != 2147483646 { return 1; }
  if min_int + 1 != -2147483647 { return 2; }
  var a: Int = 1000000;
  var b: Int = 2000000;
  var c: Int = a * b;
  if c != 2000000000000 { return 3; }
  var d: Int = 100;
  var e: Int = d * d * d;
  if e != 1000000 { return 4; }
  var f: Int = max_int / 2;
  if f * 2 + 1 != max_int { return 5; }
  var zero: Int = 0;
  var neg_one: Int = zero - 1;
  if neg_one != -1 { return 6; }
  var shift: Int = 1;
  var i: Int = 0;
  while i < 10 { shift = shift * 2; i = i + 1; }
  if shift != 1024 { return 7; }
  return 0;
}
