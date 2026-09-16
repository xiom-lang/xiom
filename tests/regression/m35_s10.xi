// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-S10: Split by delimiter -- find delimiter positions and extract parts
use stdlib.xiom.string;
fn first_split(s: Str, delim: Char) -> Str {
  if s.len() == 0 { return ""; }
  var del_byte = delim as Int;
  var i: Int = 0;
  while i < s.len() {
    if (string.byte_at(s, i) as Int) == del_byte {
      return string.str_slice(s, 0, i);
    }
    i = i + 1;
  }
  return s;
}
fn main() -> Int {
  if first_split("hello,world", ',') == "hello" && first_split("abc", ',') == "abc" && first_split(",xyz", ',') == "" { return 0; }
  return 1;
}
