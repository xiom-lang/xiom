// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m37_inline_call_concat
// BUG 22 #11 regression: method-call results used INLINE as concat
// operands. The old `"len = " + v.len()` inttoptr'd the length (garbage
// pointer -> AV); binding to a var first worked. The explicit
// convert.int_to_string keeps that formatting path covered now that the
// checker rejects implicit Int + Str.

use xiom.io;
use xiom.convert;

fn main() -> Int {
  var v = Vec[Int].new();
  v.push(1); v.push(2);
  var s = "len = " + convert.int_to_string(v.len());
  if s != "len = 2" { io.println("got: " + s); return 1; }
  // Inline call in arithmetic on the RHS of concat
  var s2 = "sum = " + convert.int_to_string(v[0] + v[1]);
  if s2 != "sum = 3" { return 2; }
  // Inline module-qualified call result in concat
  var s3 = "len2 = " + convert.int_to_string(m37_inline_call_concat._len(v));
  if s3 != "len2 = 2" { return 3; }
  return 0;
}

pub fn _len(v: Vec[Int]) -> Int { return v.len(); }
