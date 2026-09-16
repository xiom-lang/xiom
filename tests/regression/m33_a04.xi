// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-A04: Array of Int -- creation and sum of all elements
fn main() -> Int {
  var arr = [7, 14, 21, 28];
  var sum: Int = 0;
  var i: Int = 0;
  while i < 4 {
    sum += arr[i];
    i += 1;
  }
  if sum == 70 { return 0; }
  return 1;
}
