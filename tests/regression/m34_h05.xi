// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-H05: Float64->Int truncation -- fractional part discarded
fn main() -> Int {
  var f: Float64 = 3.7;
  var i: Int = f as Int;
  var g: Float64 = -3.7;
  var j: Int = g as Int;
  if i == 3 && j == -3 { return 0; }
  return 1;
}
