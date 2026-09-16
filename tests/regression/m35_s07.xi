// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-S07: String to uppercase -- build uppercase via ASCII char mapping
use stdlib.xiom.string;
fn to_upper(s: Str) -> Str {
  var result: Str = "";
  var i: Int = 0;
  while i < s.len() {
    var b = string.byte_at(s, i);
    if b >= 97 && b <= 122 {
      var pos: Int = (b as Int) - 97;
      var upper_alpha: Str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
      result = result + string.str_slice(upper_alpha, pos, pos + 1);
    } else {
      result = result + string.str_slice(s, i, i + 1);
    }
    i = i + 1;
  }
  return result;
}
fn main() -> Int {
  if to_upper("hello") == "HELLO" && to_upper("") == "" && to_upper("aBc") == "ABC" { return 0; }
  return 1;
}
