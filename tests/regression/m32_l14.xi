// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-L14: While with counter -- track iterations through loop
fn main() -> Int {
  var x: Int = 128;
  var steps: Int = 0;
  while x > 1 {
    x /= 2;
    steps += 1;
  }
  // 128->64->32->16->8->4->2->1: 7 divisions
  if steps == 7 && x == 1 { return 0; }
  return 1;
}
