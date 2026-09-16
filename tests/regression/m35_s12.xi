// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-S12: String comparison -- character-by-character lexicographic order
use stdlib.xiom.string;
fn strcmp(a: Str, b: Str) -> Int {
  var i: Int = 0;
  while i < a.len() && i < b.len() {
    var ba = string.byte_at(a, i);
    var bb = string.byte_at(b, i);
    if ba < bb { return -1; }
    if ba > bb { return 1; }
    i = i + 1;
  }
  if a.len() < b.len() { return -1; }
  if a.len() > b.len() { return 1; }
  return 0;
}
fn main() -> Int {
  if strcmp("abc", "abc") == 0 && strcmp("abc", "abd") == -1 && strcmp("abd", "abc") == 1 && strcmp("ab", "abc") == -1 { return 0; }
  return 1;
}
