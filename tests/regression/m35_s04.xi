// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-S04: Count character occurrences -- loop and count matching bytes
use stdlib.xiom.string;
fn count_char(s: Str, ch: Char) -> Int {
  var count: Int = 0;
  var target = ch as Int;
  var i: Int = 0;
  while i < s.len() {
    if (string.byte_at(s, i) as Int) == target { count = count + 1; }
    i = i + 1;
  }
  return count;
}
fn main() -> Int {
  if count_char("hello", 'l') == 2 && count_char("aaa", 'a') == 3 && count_char("abc", 'z') == 0 && count_char("", 'x') == 0 { return 0; }
  return 1;
}
