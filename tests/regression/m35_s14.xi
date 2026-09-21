// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-S14: String suffix check -- verify s ends with suffix character-by-character
use stdlib.xiom.string;
fn ends_with(s: Str, suffix: Str) -> Bool {
  if suffix.len() > s.len() { return false; }
  if suffix.len() == 0 { return true; }
  var offset: Int = s.len() - suffix.len();
  var i: Int = 0;
  while i < suffix.len() {
    if string.byte_at(s, offset + i) != string.byte_at(suffix, i) { return false; }
    i = i + 1;
  }
  return true;
}
fn main() -> Int {
  if ends_with("hello world", "world") && ends_with("abc", "") && !ends_with("abc", "abd") && ends_with("abc", "bc") { return 0; }
  return 1;
}
