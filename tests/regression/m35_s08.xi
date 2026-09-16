// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-S08: String to lowercase -- build lowercase via ASCII char mapping
use stdlib.xiom.string;
fn to_lower(s: Str) -> Str {
  var result: Str = "";
  var i: Int = 0;
  while i < s.len() {
    var b = string.byte_at(s, i);
    if b >= 65 && b <= 90 {
      var pos: Int = (b as Int) - 65;
      var lower_alpha: Str = "abcdefghijklmnopqrstuvwxyz";
      result = result + string.str_slice(lower_alpha, pos, pos + 1);
    } else {
      result = result + string.str_slice(s, i, i + 1);
    }
    i = i + 1;
  }
  return result;
}
fn main() -> Int {
  if to_lower("HELLO") == "hello" && to_lower("") == "" && to_lower("AbC") == "abc" { return 0; }
  return 1;
}
