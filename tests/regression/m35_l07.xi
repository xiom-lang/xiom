// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-L07: Struct with array -- struct containing array-like access via separate fields
type DataBlock = { count: Int; tag: Char; }

fn main() -> Int {
  var arr = [DataBlock{ count: 1; tag: 'A'; }, DataBlock{ count: 2; tag: 'B'; }, DataBlock{ count: 3; tag: 'C'; }];
  var sum: Int = 0;
  var i: Int = 0;
  while i < 3 {
    sum += arr[i].count;
    i += 1;
  }
  if sum != 6 { return 1; }
  if arr[0].tag != 'A' { return 2; }
  if arr[2].tag != 'C' { return 3; }
  return 0;
}
