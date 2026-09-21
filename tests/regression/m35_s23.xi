// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-S23: Check anagrams -- compare character frequency arrays
use stdlib.xiom.string;
fn char_count(s: Str, ch: Char) -> Int {
  var count: Int = 0;
  var t_byte = ch as Int;
  var i: Int = 0;
  while i < s.len() {
    if (string.byte_at(s, i) as Int) == t_byte { count = count + 1; }
    i = i + 1;
  }
  return count;
}
fn is_anagram(a: Str, b: Str) -> Bool {
  if a.len() != b.len() { return false; }
  var i: Int = 0;
  while i < a.len() {
    var ch_byte = string.byte_at(a, i);
    var ch: Char = ch_byte as Char;
    if char_count(a, ch) != char_count(b, ch) { return false; }
    i = i + 1;
  }
  return true;
}
fn main() -> Int {
  if is_anagram("listen", "silent") && is_anagram("", "") && !is_anagram("hello", "world") && is_anagram("abc", "cba") { return 0; }
  return 1;
}
