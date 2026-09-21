// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-H19: Cast after arithmetic with overflow check -- detect truncation loss
fn main() -> Int {
  var a: Int16 = 100;
  var b: Int16 = 200;
  var sum: Int16 = a + b;
  var narrow: Int8 = sum as Int8;
  var wide: Int16 = narrow as Int16;
  if wide == 44 as Int16 { return 0; }
  return 1;
}
