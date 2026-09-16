// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-E23: All operators in one expression
fn main() -> Int {
  var a = 10; var b = 3; var c = 5; var d = 2;
  var result = a + b * c - d / 1 + a % b;
  if result != 24 { return 1; }
  var cmp = a > b && b < c || c == d + 3 && d != 0;
  if !cmp { return 2; }
  var neg = -a + b;
  if neg != -7 { return 3; }
  return 0;
}
