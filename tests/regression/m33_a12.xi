// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-A12: Array bounds checking -- verify all indices accessible within array
fn main() -> Int {
  var arr = [5, 10, 15, 20, 25];
  var n: Int = 5;
  var sum: Int = 0;
  var i: Int = 0;
  while i < n {
    sum += arr[i];
    i += 1;
  }
  if sum == 75 { return 0; }
  return 1;
}
