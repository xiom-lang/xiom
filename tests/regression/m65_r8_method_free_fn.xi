// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// R8 regression: method-position FREE-FN calls (receiver sugar) and
// contracts over them.
// - `s.char_count_local()` where `char_count_local(s: Str)` is a free fn:
//   the checker used to reject it ("cannot call ... on this expression")
//   and catalog bodies compiled it to a constant-0 stub, so the json
//   char_at ensures evaluated 0 and aborted every Some return.
// - A free fn's `ensures` using the same sugar must evaluate the real
//   value (`s.len()`), and the builtin Char-returning `.char_at` path must
//   stay coherent with such clauses present (verified against a patched
//   stdlib copy: smoke_string_glob, convert_url, ascii85, json all green).
module m65_r8_method_free_fn

use xiom.io;
use xiom.string;

fn char_count_local(s: Str) -> Int {
  return s.len();
}

fn pick(s: Str, pos: Int) -> Int
  ensures: result <= s.char_count_local()
{
  if pos < 0 || pos >= s.len() { return 0; }
  return pos;
}

fn main() -> Int {
  let s = "abc";
  if s.char_count_local() != 3 {
    io.println("r8 count bad");
    return 1;
  }
  if pick("abc", 1) != 1 {
    io.println("r8 contract bad");
    return 2;
  }
  let c = s.char_at(1);
  if c != 'b' {
    io.println("r8 char_at bad");
    return 3;
  }
  io.println("r8 ok");
  return 0;
}
