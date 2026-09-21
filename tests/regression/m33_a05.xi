// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-A05: Array of Float64 -- creation and sum
fn main() -> Int {
  var arr = [1.5, 2.5, 3.5, 4.5];
  var sum: Float64 = 0.0;
  var i: Int = 0;
  while i < 4 {
    sum += arr[i];
    i += 1;
  }
  if sum == 12.0 { return 0; }
  return 1;
}
