// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-S06: Replace substring -- find and replace first occurrence
use stdlib.xiom.string;
fn replace_first(s: Str, from: Str, to: Str) -> Str {
  if from.len() == 0 { return s; }
  var i: Int = 0;
  while i <= s.len() - from.len() {
    var j: Int = 0;
    var match_found: Bool = true;
    while j < from.len() {
      if string.byte_at(s, i + j) != string.byte_at(from, j) { match_found = false; }
      j = j + 1;
    }
    if match_found {
      var prefix: Str = string.str_slice(s, 0, i);
      var suffix: Str = string.str_slice(s, i + from.len(), s.len());
      return prefix + to + suffix;
    }
    i = i + 1;
  }
  return s;
}
fn main() -> Int {
  var r: Str = replace_first("hello world", "world", "there");
  if r == "hello there" && replace_first("abc", "x", "y") == "abc" && replace_first("aaa", "a", "b") == "baa" { return 0; }
  return 1;
}
