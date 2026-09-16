// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M57: nested &Vec[Vec[Float64]] PARAM reads -- the BUG 57 regression.
// Module fns taking &Vec[Vec[Float64]] read garbage through chained
// indexes (raw m[0][1], defensive-copy rows, unary minus on nested index).
// Fixed by threading element types through chained indexes (indexed_elem_
// types map) + full generic names in vec_elem_from_type_annotation.
module m57_geom_nested_param

fn identity2() -> Vec[Vec[Float64]] {
  var out = Vec[Vec[Float64]].new();
  var r0 = Vec[Float64].new();
  r0.push(1.0);
  r0.push(0.0);
  var r1 = Vec[Float64].new();
  r1.push(0.0);
  r1.push(1.0);
  out.push(r0);
  out.push(r1);
  return out;
}

fn trace2(a: &Vec[Vec[Float64]]) -> Float64 {
  var ac = Vec[Vec[Float64]].new();
  var i = 0;
  while i < a.len() {
    ac.push(a[i]);
    i = i + 1;
  }
  var s = 0.0;
  var k = 0;
  while k < ac.len() {
    s = s + ac[k][k];
    k = k + 1;
  }
  return s;
}

fn read01(m: &Vec[Vec[Float64]]) -> Float64 {
  return m[0][1];
}

fn main() -> Int {
  var id = identity2();
  if id[0][0] != 1.0 { return 10; }
  if id[1][1] != 1.0 { return 11; }
  var t = trace2(&id);
  if t != 2.0 { return 12; }
  var d = read01(&id);
  if d != 0.0 { return 13; }
  // skew matrix: raw param reads + negated nested index compare
  var sk = Vec[Vec[Float64]].new();
  var q0 = Vec[Float64].new();
  q0.push(0.0);
  q0.push(-3.0);
  var q1 = Vec[Float64].new();
  q1.push(3.0);
  q1.push(0.0);
  sk.push(q0);
  sk.push(q1);
  if read01(&sk) != -3.0 { return 14; }
  var neg = -(sk[0][1]);
  if neg != 3.0 { return 15; }
  return 0;
}
