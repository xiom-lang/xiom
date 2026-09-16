// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-D13: Circular buffer -- wraparound pattern
fn main() -> Int {
  var b0: Int = 0;
  var b1: Int = 0;
  var b2: Int = 0;
  var b3: Int = 0;
  var head: Int = 0;
  b0 = 10;
  b1 = 20;
  b2 = 30;
  b3 = 40;
  head = 1;
  if b1 != 20 { return 1; }
  head = 2;
  if b2 != 30 { return 2; }
  head = 3;
  if b3 != 40 { return 3; }
  head = 0;
  if b0 != 10 { return 4; }
  return 0;
}
