// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M57b: Vec[Vec[Float64]] LOCAL ctor element registration -- the BUG 57
// follow-up regression. `var basis = Vec[Vec[Float64]].new();` registered
// the element type DOUBLE-wrapped ("Vec[Vec[Float64]]" -- the nested-ctor
// arm re-wrapped the outer Vec around the inner type arg). Chained inner
// reads (basis[u][jj]) that compile BEFORE the first basis.push() (single-
// pass compilation: reads in iteration bodies precede the push at the end
// of the loop) then memcpy'd a whole %struct.Vec out of an 8-byte double
// slot, and the struct->scalar coercion emitted "store %struct.Vec <i64>"
// (invalid IR; clang: "defined with type i64 but expected %struct.Vec").
// Fixed by rendering the INNER type arg only (type_arg_to_name(idx)), so
// the element registers as "Vec[Float64]" and inner reads take the typed
// float path. Source ordering matters: reads must precede the push.
module m57b_geom_local_nested

fn gram_schmidt_like() -> Float64 {
  var basis = Vec[Vec[Float64]].new();
  var s = 0.0;
  var i = 0;
  while i < 2 {
    var v = Vec[Float64].new();
    var j = 0;
    while j < 2 {
      if i == 0 {
        v.push(1.0 + j as Float64);
      } else {
        v.push(3.0 + j as Float64);
      }
      j = j + 1;
    }
    // Chained inner reads over basis[u][jj] -- compiled BEFORE the
    // basis.push(v) below. Runtime: basis is empty for i == 0, holds one
    // row for i == 1 (proj = dot(v, basis[0]) = 3*1 + 4*2 = 11).
    var proj = 0.0;
    var u = 0;
    while u < basis.len() {
      var jj = 0;
      while jj < 2 {
        proj = proj + v[jj] * basis[u][jj];
        jj = jj + 1;
      }
      u = u + 1;
    }
    var kk = 0;
    while kk < basis.len() {
      var jj = 0;
      while jj < 2 {
        v[jj] = v[jj] - proj * basis[kk][jj];
        jj = jj + 1;
      }
      kk = kk + 1;
    }
    basis.push(v);
    s = s + v[0] + v[1];
    i = i + 1;
  }
  return s;
}

fn main() -> Int {
  var s = gram_schmidt_like();
  // i = 0: basis empty -> v stays [1, 2]; push -> basis = [[1, 2]]
  // i = 1: v = [3, 4]; proj = 3*1 + 4*2 = 11
  //        v = [3, 4] - 11 * [1, 2] = [-8, -18]
  // s = (1 + 2) + (-8 + -18) = 3 - 26 = -23
  if s != -23.0 { return 20; }
  return 0;
}
