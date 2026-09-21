// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m37_vec_f64
// BUG 12/17 regression: Vec[Float64] and Vec[Str] element reads through
// &Vec[T] PARAMS (catalog style) must preserve the element type -- Float64
// loads double (not i64+sitofp of the bit pattern), Str compares via
// strcmp (not pointer compare).

fn sum_first(v: &Vec[Float64]) -> Float64 {
  var x = v[0];
  return x;
}

fn eq_strs(ga: &Vec[Str], gb: &Vec[Str]) -> Bool {
  return ga[0] == gb[0];
}

fn main() -> Int {
  var v = Vec[Float64].new();
  v.push(3.14159);
  var x = sum_first(&v);
  if x != 3.14159 { return 1; }
  var ga = Vec[Str].new();
  ga.push("he");
  var gb = Vec[Str].new();
  gb.push("he");
  if !eq_strs(&ga, &gb) { return 2; }
  if ga[0] != "he" { return 3; }
  return 0;
}
