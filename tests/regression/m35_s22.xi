// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-S22: Capitalize first letter -- convert first char to uppercase if lowercase
use stdlib.xiom.string;
fn capitalize(s: Str) -> Str {
  if s.len() == 0 { return ""; }
  var b = string.byte_at(s, 0);
  if b >= 97 && b <= 122 {
    var pos: Int = (b as Int) - 97;
    var upper_alpha: Str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    return string.str_slice(upper_alpha, pos, pos + 1) + string.str_slice(s, 1, s.len());
  }
  return s;
}
fn main() -> Int {
  if capitalize("hello") == "Hello" && capitalize("Hello") == "Hello" && capitalize("") == "" && capitalize("a") == "A" { return 0; }
  return 1;
}
