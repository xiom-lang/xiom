// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-S16: Find last occurrence -- return index of last match or -1
use stdlib.xiom.string;
fn last_index_of(s: Str, ch: Char) -> Int {
  var target = ch as Int;
  var i = s.len() - 1;
  while i >= 0 {
    if (string.byte_at(s, i) as Int) == target { return i; }
    i = i - 1;
  }
  return -1;
}
fn main() -> Int {
  if last_index_of("hello", 'l') == 3 && last_index_of("hello", 'h') == 0 && last_index_of("hello", 'z') == -1 && last_index_of("", 'a') == -1 { return 0; }
  return 1;
}
