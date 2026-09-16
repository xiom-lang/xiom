// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m37_inline_call_concat
// BUG 22 #11 regression: method-call results used INLINE as concat
// operands. "len = " + v.len() inttoptr'd the length (garbage pointer ->
// AV); binding to a var first worked. The concat must format the call
// result via xiom_int_to_string.

use xiom.io;

fn main() -> Int {
  var v = Vec[Int].new();
  v.push(1); v.push(2);
  var s = "len = " + v.len();
  if s != "len = 2" { io.println("got: " + s); return 1; }
  // Inline call in arithmetic on the RHS of concat
  var s2 = "sum = " + (v[0] + v[1]);
  if s2 != "sum = 3" { return 2; }
  // Inline module-qualified call result in concat
  var s3 = "len2 = " + m37_inline_call_concat._len(v);
  if s3 != "len2 = 2" { return 3; }
  return 0;
}

pub fn _len(v: Vec[Int]) -> Int { return v.len(); }
