// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

fn xiom_char_at(src: Int, pos: Int) -> Int;

fn test(src: Int) -> Int {
  var pos = 0;
  var done = 1 == 0;
  while !(done) {
    if (pos >= 10) { done = 1 == 1; }
    else {
      var c = xiom_char_at(&src, &pos);
      if c == 42 { done = 1 == 1; }
    }
    if !(done) { pos = pos + 1; }
  }
  var x = pos;
  return x;
}
fn main() -> Int { return 0; }
