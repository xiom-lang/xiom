// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-S20: Longest word -- find word with maximum length
use stdlib.xiom.string;
fn longest_word_len(s: Str) -> Int {
  if s.len() == 0 { return 0; }
  var max_len: Int = 0;
  var curr_len: Int = 0;
  var space: UInt8 = 32;
  var i: Int = 0;
  while i < s.len() {
    if string.byte_at(s, i) != space {
      curr_len = curr_len + 1;
    } else {
      if curr_len > max_len { max_len = curr_len; }
      curr_len = 0;
    }
    i = i + 1;
  }
  if curr_len > max_len { max_len = curr_len; }
  return max_len;
}
fn main() -> Int {
  if longest_word_len("the quick brown fox") == 5 && longest_word_len("a bb ccc") == 3 && longest_word_len("") == 0 { return 0; }
  return 1;
}
