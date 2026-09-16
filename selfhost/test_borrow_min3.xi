// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

fn xiom_str_len(src: Int) -> Int;
fn xiom_char_at(src: Int, pos: Int) -> Int;

fn test(src: Int) -> Int {
  var len = xiom_str_len(&src);
  var pos = 0;
  while (pos + 2 < len) {
    var c0 = xiom_char_at(&src, &pos);
    var c1 = xiom_char_at(&src, &pos + 1);
    if c0 == 102 && c1 == 110 {
      var c2 = xiom_char_at(&src, &pos + 2);
      if c2 == 32 || c2 == 9 {
        pos = pos + 3;
        var ws_done = 1 == 0;
        while !(ws_done) {
          if (pos >= len) { ws_done = 1 == 1; }
          else { var ws_c = xiom_char_at(&src, &pos);
            if ws_c != 32 { ws_done = 1 == 1; }
          }
          if !(ws_done) { pos = pos + 1; }
        }
        var name_start = pos + 0;
        var body_start = pos;
      }
    }
    pos = pos + 1;
  }
  return 0;
}
fn main() -> Int { return 0; }
