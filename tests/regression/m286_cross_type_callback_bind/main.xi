// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m286 lock (REL-20261010-1549-stdlib-7): a cross-type generic callback
// (`mapv[Int, Str](&v, to_s)`) returned a Vec whose element type was lost --
// the declared "Vec[U]" was dropped at the binding, or seeded as the raw
// placeholder "U", so `ws[0]` crossed as a scalar and the Str handle was
// byte-materialized in the comparison. Both the inferred and the explicit
// bracket spellings must keep the substituted "Vec[Str]".
module m286_cross_type_callback_bind.main

use xiom.string;
use xiom.convert;

fn mapv[T, U](v: &Vec[T], f: fn(T) -> U) -> Vec[U] {
  var out = Vec[U].new();
  var i = 0;
  while i < v.len() {
    out.push(f(v[i]));
    i = i + 1;
  }
  return out;
}

fn to_s(x: Int) -> Str { return convert.int_to_string(x); }

fn main() -> Int {
  var v: Vec[Int] = Vec[Int].new();
  v.push(1);
  v.push(2);
  // Inferred type args (the binding keeps "Vec[Str]").
  let ws = mapv(&v, to_s);
  if ws.len() != 2 { return 40; }
  if str_compare(ws[0], "1") != 0 { return 41; }
  if str_compare(ws[1], "2") != 0 { return 42; }
  // Explicit bracket spelling (Call(Index(Ident, ...))).
  let ws2 = mapv[Int, Str](&v, to_s);
  if ws2.len() != 2 { return 43; }
  if str_compare(ws2[0], "1") != 0 { return 44; }
  if str_compare(ws2[1], "2") != 0 { return 45; }
  return 0;
}
