// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-L05: While loop over array -- sum elements [10, 20, 30, 40, 50]
fn main() -> Int {
  var arr = [10, 20, 30, 40, 50];
  var sum: Int = 0;
  var i: Int = 0;
  while i < 5 {
    sum += arr[i];
    i += 1;
  }
  if sum == 150 { return 0; }
  return 1;
}
