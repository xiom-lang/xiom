// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-A15: Array iteration -- while loop over array with multiple accumulators
fn main() -> Int {
  var arr = [2, 4, 6, 8, 10];
  var sum: Int = 0;
  var prod: Int = 1;
  var i: Int = 0;
  while i < 5 {
    sum += arr[i];
    prod *= arr[i];
    i += 1;
  }
  if sum == 30 && prod == 3840 { return 0; }
  return 1;
}
