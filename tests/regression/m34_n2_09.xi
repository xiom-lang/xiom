// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-N2-09: Array-of-array-of-array stress -- chained array literal and indexing
fn main() -> Int {
  var a0 = [10, 20, 30];
  var a1 = [a0[0] + 1, a0[1] + 1, a0[2] + 1];
  var a2 = [a1[0] + 1, a1[1] + 1, a1[2] + 1];
  var a3 = [a2[0] + 1, a2[1] + 1, a2[2] + 1];
  if a0[0] == 10 && a1[1] == 21 && a2[2] == 32 && a3[0] == 13 { return 0; }
  return 1;
}
