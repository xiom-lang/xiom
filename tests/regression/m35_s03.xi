// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-S03: Substring search -- manual needle-in-haystack check
use stdlib.xiom.string;
fn contains(haystack: Str, needle: Str) -> Bool {
  if needle.len() == 0 { return true; }
  if needle.len() > haystack.len() { return false; }
  var i: Int = 0;
  while i <= haystack.len() - needle.len() {
    var j: Int = 0;
    var found: Bool = true;
    while j < needle.len() {
      if string.byte_at(haystack, i + j) != string.byte_at(needle, j) { found = false; }
      j = j + 1;
    }
    if found { return true; }
    i = i + 1;
  }
  return false;
}
fn main() -> Int {
  if contains("hello world", "world") && contains("hello world", "hello") && contains("abc", "") && !contains("abc", "d") { return 0; }
  return 1;
}
