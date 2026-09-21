// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-V16: Nested Vec -- Vec[Vec[Int]] outer operations
use xiom.collections;

fn main() -> Int {
  var outer = Vec[Vec[Int]].new();
  var inner1 = Vec[Int].new();
  inner1.push(1);
  inner1.push(2);
  var inner2 = Vec[Int].new();
  inner2.push(10);
  inner2.push(20);
  inner2.push(30);
  outer.push(inner1);
  outer.push(inner2);
  if outer.len() != 2 { return 1; }
  // Remove last inner Vec
  var p = outer.remove(1);
  if outer.len() != 1 { return 2; }
  match p {
    Some(_row) => {}
    None => { return 3; }
  }
  // Insert new inner Vec
  var inner3 = Vec[Int].new();
  inner3.push(42);
  outer.insert(1, inner3);
  if outer.len() != 2 { return 4; }
  // Verify outer still has 2 elements
  var final_len = outer.len();
  if final_len != 2 { return 5; }
  return 0;
}
