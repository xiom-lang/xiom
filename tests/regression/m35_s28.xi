// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-S28: String distance -- Hamming distance between equal-length strings
use stdlib.xiom.string;
fn hamming(a: Str, b: Str) -> Int {
  if a.len() != b.len() { return -1; }
  var dist: Int = 0;
  var i: Int = 0;
  while i < a.len() {
    if string.byte_at(a, i) != string.byte_at(b, i) { dist = dist + 1; }
    i = i + 1;
  }
  return dist;
}
fn main() -> Int {
  if hamming("karolin", "kathrin") == 3 && hamming("abc", "abc") == 0 && hamming("abc", "abd") == 1 { return 0; }
  return 1;
}
