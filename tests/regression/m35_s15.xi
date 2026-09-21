// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-S15: Find first occurrence -- return index of first match or -1
use stdlib.xiom.string;
fn index_of(s: Str, ch: Char) -> Int {
  var target = ch as Int;
  var i: Int = 0;
  while i < s.len() {
    if (string.byte_at(s, i) as Int) == target { return i; }
    i = i + 1;
  }
  return -1;
}
fn main() -> Int {
  if index_of("hello", 'e') == 1 && index_of("hello", 'o') == 4 && index_of("hello", 'z') == -1 && index_of("", 'a') == -1 { return 0; }
  return 1;
}
