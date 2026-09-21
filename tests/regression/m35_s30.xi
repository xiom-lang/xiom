// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-S30: Balanced brackets -- check (), [], {} balancing using depth counters
use stdlib.xiom.string;
fn is_balanced(s: Str) -> Bool {
  var depth_paren: Int = 0;
  var depth_brack: Int = 0;
  var depth_brace: Int = 0;
  var i: Int = 0;
  while i < s.len() {
    var b = string.byte_at(s, i);
    if b == 40 { depth_paren = depth_paren + 1; }
    else {
      if b == 41 {
        depth_paren = depth_paren - 1;
        if depth_paren < 0 { return false; }
      } else {
        if b == 91 { depth_brack = depth_brack + 1; }
        else {
          if b == 93 {
            depth_brack = depth_brack - 1;
            if depth_brack < 0 { return false; }
          } else {
            if b == 123 { depth_brace = depth_brace + 1; }
            else {
              if b == 125 {
                depth_brace = depth_brace - 1;
                if depth_brace < 0 { return false; }
              }
            }
          }
        }
      }
    }
    i = i + 1;
  }
  return depth_paren == 0 && depth_brack == 0 && depth_brace == 0;
}
fn main() -> Int {
  if is_balanced("()") && is_balanced("") && !is_balanced("(") && !is_balanced(")") && is_balanced("({})[]") { return 0; }
  return 1;
}
