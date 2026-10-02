// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m170 lock (stdlib C24-2 relay, p_curve_thunk_zero): a call through an
// fn-typed PARAM must record the parameter's declared return type on the
// binding (`var v = samples(t)`). Before the fix the non-mono param
// registration stored the type with type_from_ast, which DROPS generic args
// ("Vec[Float64]" -> "Vec"), so `v[0]` inside the callee took the scalar
// i64 load path and curve_length's sampled reads collapsed (total != arc
// length). call.rs already used fn_local_returns for the emitted signature,
// and infer_call_return_xiom did not consult it at all before the fix.
module m170_fn_typed_vec_return

use xiom.geom.curves;
use xiom.geom.vector;

fn line(t: Float64) -> Vec[Float64] {
  var out = Vec[Float64].new();
  out.push(t);
  out.push(0.0);
  return out;
}

// IR-locked: the element read must bitcast the loaded double bits.
fn fn_typed_vec_read(samples: fn(Float64) -> Vec[Float64], t: Float64) -> Float64 {
  var v = samples(t);
  return v[0];
}

fn fn_typed_vec_len(samples: fn(Float64) -> Vec[Float64], t: Float64) -> Int {
  var v = samples(t);
  return v.len();
}

// Mirrors curve_length's body: two calls, len, element reads in a loop.
fn fn_typed_poly_len(samples: fn(Float64) -> Vec[Float64], n: Int) -> Float64 {
  var total = 0.0;
  var prev = samples(0.0);
  var i = 1;
  while i <= n {
    var cur = samples(i as Float64);
    var s = 0.0;
    var k = 0;
    var m = prev.len();
    if cur.len() < m { m = cur.len(); }
    while k < m {
      var d = cur[k] - prev[k];
      s = s + d * d;
      k = k + 1;
    }
    total = total + s;
    prev = cur;
    i = i + 1;
  }
  return total;
}

// Option[Vec[Float64]] payload construction + extraction (C24-2b): the
// erased Option's i64 payload slot must hold the boxed handle that
// `.unwrap()` dereferences.
fn maybe_vec(t: Float64) -> Option[Vec[Float64]] {
  var v = Vec[Float64].new();
  v.push(t);
  v.push(0.0);
  return Some(v);
}

fn main() -> Int {
  // Control: the same function value called directly is correct.
  var s = line(0.5);
  if s[0] != 0.5 { return 1; }

  // Catalog: curve_length's calls through the fn-typed parameter.
  if curves.curve_length(line, 0.0, 1.0, 2) != 1.0 { return 2; }

  // User-space replicas of the callee-side shapes.
  if fn_typed_vec_read(line, 0.5) != 0.5 { return 3; }
  if fn_typed_vec_len(line, 0.5) != 2 { return 4; }
  if fn_typed_poly_len(line, 1) != 1.0 { return 5; }

  // Option[Vec[Float64]]: user-space Some(v) + unwrap.
  var r = maybe_vec(0.5);
  if !r.is_some { return 6; }
  var rv = r.unwrap();
  if rv.len() != 2 { return 7; }
  if rv[0] != 0.5 { return 8; }

  // Option[Vec[Float64]]: catalog literal (refract) + unwrap.
  var va = Vec[Float64].new();
  va.push(1.0);
  va.push(0.0);
  var vn = Vec[Float64].new();
  vn.push(0.0);
  vn.push(1.0);
  var rf = vector.refract(&va, &vn, 1.0);
  if !rf.is_some { return 9; }
  var rr = rf.unwrap();
  if rr.len() != 2 { return 10; }
  if rr[0] != 1.0 { return 11; }
  if rr[1] != 0.0 { return 12; }
  return 0;
}
