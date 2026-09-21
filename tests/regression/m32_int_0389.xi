// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int8 boundary compare: -128 < 0 < 127
fn main() -> Int {
  var min: Int8 = -128 as Int8;
  var zero: Int8 = 0;
  var max: Int8 = 127;
  if min < zero && zero < max { return 0; }
  return 1;
}
