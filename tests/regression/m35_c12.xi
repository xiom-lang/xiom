// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-C12: for-in pattern using while + index -- simulate for loop over array-like logic
fn sum_array_elements(count: Int) -> Int {
  var total: Int = 0;
  var idx: Int = 0;
  while idx < count { total = total + idx; idx = idx + 1; }
  return total;
}
fn main() -> Int {
  if sum_array_elements(0) != 0 { return 1; }
  if sum_array_elements(1) != 0 { return 2; }
  if sum_array_elements(5) != 10 { return 3; }
  if sum_array_elements(11) != 55 { return 4; }
  return 0;
}
