// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-E27: Deep array indexing -- chained array index computations
fn main() -> Int {
  var arr = [10, 20, 30, 40, 50, 60, 70, 80, 90, 100];
  var i = 0;
  var j = arr[i];
  if j != 10 { return 1; }
  var k = arr[1 + 2];
  if k != 40 { return 2; }
  var m = arr[(arr[0] + arr[1]) / 10];
  if m != 40 { return 3; }
  var n = arr[arr[3] / 10 - 1];
  if n != 40 { return 4; }
  var p = arr[9];
  if p != 100 { return 5; }
  return 0;
}