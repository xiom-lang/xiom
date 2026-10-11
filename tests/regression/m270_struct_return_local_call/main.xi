// m270 (REL-20261010-1854-pulse): consumer with a same-named foreign
// `out_text` returning Str. `mod_a.handle` must still call MOD_A's own
// struct-returning `out_text`; pre-m261 the bare slot hijacked it and the
// struct return was misread (the pulse site-mode startup AV class).
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m270_consumer

use mod_a;

fn out_text(s: Str) -> Str {
  return s;
}

fn main() -> Int {
  let r = mod_a.handle(41);
  if r.code != 42 {
    return 1;
  }
  if r.body != "x" {
    return 2;
  }
  if out_text("ok") != "ok" {
    return 3;
  }
  return 0;
}
