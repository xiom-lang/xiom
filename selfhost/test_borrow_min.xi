// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

fn xiom_str_len(src: Int) -> Int;
fn xiom_char_at(src: Int, pos: Int) -> Int;

fn test(src: Int) -> Int {
  var pos = 0;
  var len = xiom_str_len(src);
  while (pos + 2 < len) {
    var c = xiom_char_at(&src, &pos);
    if c == 102 {
      pos = pos + 1;
      var done = 1 == 0;
      while !(done) {
        if (pos >= len) { done = 1 == 1; }
        else {
          var c2 = xiom_char_at(&src, &pos);
          if c2 == 123 { done = 1 == 1; }
        }
        if !(done) { pos = pos + 1; }
      }
      var body_start = pos;
    }
    pos = pos + 1;
  }
  return 0;
}
fn main() -> Int { return 0; }
