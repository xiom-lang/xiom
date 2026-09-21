// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-S04: Struct with all primitive types (Int, Float64, Bool, Char, Str)
type Record = { i: Int; f: Float64; b: Bool; c: Char; s: Str; }
fn main() -> Int {
  var r = Record{ i: 42; f: 3.14; b: true; c: 'X'; s: "hello"; };
  var ok: Bool = true;
  if r.i != 42 { ok = false; }
  if r.f != 3.14 { ok = false; }
  if !r.b { ok = false; }
  if r.c != 'X' { ok = false; }
  if ok { return 0; }
  return 1;
}
