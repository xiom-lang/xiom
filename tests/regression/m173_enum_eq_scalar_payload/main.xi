// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m173 guard for finding (g): scalar and Str payloads are lowerable and MUST
// keep compiling and comparing correctly -- the rejection only covers
// aggregate payloads.
enum Val {
  I(v: Int),
  S(v: Str),
  N,
}

fn main() -> Int {
  var a = Val.I(7);
  var b = Val.I(7);
  var c = Val.I(8);
  if !(a == b) { return 11; }
  if a == c { return 12; }
  if a != b { return 13; }

  var s1 = Val.S("x");
  var s2 = Val.S("x");
  var s3 = Val.S("y");
  if !(s1 == s2) { return 14; }
  if s1 == s3 { return 15; }

  var n1 = Val.N;
  var n2 = Val.N;
  if !(n1 == n2) { return 16; }
  if n1 == a { return 17; }
  return 0;
}
