// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-D08: Max-heap -- bubble-up verify
fn main() -> Int {
  var d0: Int = 50;
  var d1: Int = 30;
  var d2: Int = 10;
  if d0 < d1 { return 1; }
  if d0 < d2 { return 2; }
  if d1 < d2 { return 3; }
  return 0;
}
