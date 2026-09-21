// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-S27: Rotation check -- check if one string is a rotation of another
use stdlib.xiom.string;
fn seems_rotation(a: Str, b: Str) -> Bool {
  if a.len() != b.len() { return false; }
  if a.len() == 0 { return true; }
  var doubled: Str = a + a;
  var i: Int = 0;
  while i <= a.len() {
    var j: Int = 0;
    var match_found: Bool = true;
    while j < b.len() {
      if string.byte_at(doubled, i + j) != string.byte_at(b, j) { match_found = false; }
      j = j + 1;
    }
    if match_found { return true; }
    i = i + 1;
  }
  return false;
}
fn main() -> Int {
  if seems_rotation("abcde", "cdeab") && seems_rotation("abcde", "eabcd") && !seems_rotation("abc", "def") && seems_rotation("", "") { return 0; }
  return 1;
}
