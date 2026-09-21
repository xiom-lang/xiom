// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-S19: Character frequency count -- count occurrences of each char in string
use stdlib.xiom.string;
fn freq_count(s: Str, target: Char) -> Int {
  var count: Int = 0;
  var t_byte = target as Int;
  var i: Int = 0;
  while i < s.len() {
    if (string.byte_at(s, i) as Int) == t_byte { count = count + 1; }
    i = i + 1;
  }
  return count;
}
fn main() -> Int {
  if freq_count("banana", 'a') == 3 && freq_count("banana", 'n') == 2 && freq_count("banana", 'b') == 1 && freq_count("banana", 'z') == 0 { return 0; }
  return 1;
}
