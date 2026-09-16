// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-A04: Find minimum value -- scan array and verify min index computation
fn main() -> Int {
  var arr = [29, 10, 14, 37, 13];
  var n: Int = 5;
  var min_idx: Int = 0;
  var j: Int = 1;
  while j < n { if arr[j] < arr[min_idx] { min_idx = j; } j = j + 1; }
  if min_idx != 1 { return 1; }
  if arr[min_idx] != 10 { return 2; }
  var arr2 = [5, 3, 8, 1, 9, 2];
  var n2: Int = 6;
  var min_idx2: Int = 0;
  j = 1;
  while j < n2 { if arr2[j] < arr2[min_idx2] { min_idx2 = j; } j = j + 1; }
  if min_idx2 != 3 { return 3; }
  return 0;
}
