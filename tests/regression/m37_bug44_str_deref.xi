// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// BUG 44 regression: deref/coercion of &Str must load i8* (the pointer),
// not i8 (a byte). Also &Str -> Str auto-coercion must deref to the VALUE.
module m37_bug44_str_deref

fn read_via_deref(s: &Str) -> Str {
  var d = *s;
  return d;
}

fn expect_str(s: Str) -> Str { return s; }

fn main() -> Int {
  var s = "hello";
  var p = &s;
  var d = *p;
  if d != "hello" { return 1; }
  var e = expect_str(p);
  if e != "hello" { return 2; }
  var f = read_via_deref(p);
  if f != "hello" { return 3; }
  var g = expect_str(&s);
  if g != "hello" { return 4; }
  var s2 = "world";
  var p2 = &s2;
  var d2 = *p2;
  if d2 != "world" { return 5; }
  return 0;
}
