// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m169 lock (stdlib C24-1 relay, p_geom_vector_result_bits): caller-side
// element reads of module-qualified RESULTS whose LEAF is shared with a
// different-return-type function must keep the callee's declared container
// type. Before the fix, `vector.lerp/clamp/hadamard` and `curves.b_spline`
// bindings lost their Vec element type (the `{receiver}.{leaf}` key missed --
// the catalog stores the resolved `geom.vector.lerp` form -- and the
// ambiguous leaf-suffix fallback returned None), so `lp[0]` took the scalar
// i64 load path: the stored double's IEEE bits surfaced as an integer
// converted to Float64 (1.5 read as 4.609e18). Unique-leaf siblings
// (cross, bezier_quad) were unaffected. The same-leaf nested sibling
// matrix.hadamard must still bind its own Vec[Vec[Float64]].
module m169_sameleaf_qualified_result_elem

use xiom.geom.vector;
use xiom.geom.curves;
use xiom.geom.matrix;
use xiom.math;

fn mk2(a: Float64, b: Float64) -> Vec[Float64] {
  var out = Vec[Float64].new();
  out.push(a);
  out.push(b);
  return out;
}

fn mk3(a: Float64, b: Float64, c: Float64) -> Vec[Float64] {
  var out = Vec[Float64].new();
  out.push(a);
  out.push(b);
  out.push(c);
  return out;
}

fn mk_row(a: Float64, b: Float64) -> Vec[Vec[Float64]] {
  var out = Vec[Vec[Float64]].new();
  out.push(mk2(a, b));
  out.push(mk2(b, a));
  return out;
}

fn main() -> Int {
  var a = mk2(1.0, 2.0);
  var b = mk2(4.0, 6.0);

  // Control: unique leaf, same caller-read shape.
  var c1 = mk3(1.0, 2.0, 3.0);
  var c2 = mk3(4.0, 5.0, 6.0);
  var x = vector.cross(&c1, &c2);
  if x[0] != -3.0 { return 1; }

  // Control: the scalar same-leaf function keeps its Float64 return.
  var m = math.lerp(0.0, 4.0, 0.5);
  if m != 2.0 { return 2; }

  // vector.lerp: shared leaf with math.lerp (Float64).
  var lp = vector.lerp(&a, &b, 0.5);
  if lp[0] != 2.5 { return 3; }
  if lp[1] != 4.0 { return 4; }

  // vector.clamp: shared leaf with math.clamp (Float64).
  var cl = vector.clamp(&b, 0.0, 5.0);
  if cl[0] != 4.0 { return 5; }
  if cl[1] != 5.0 { return 6; }

  // vector.hadamard: shared leaf with matrix.hadamard (nested Vec).
  var hd = vector.hadamard(&a, &b);
  if hd[0] != 4.0 { return 7; }
  if hd[1] != 12.0 { return 8; }

  // curves.b_spline: shared leaf with geometry_extended.b_spline (Vec2);
  // constant control points make every B-spline weight sum to 1.
  var cps = Vec[Vec[Float64]].new();
  cps.push(mk2(1.5, 0.0));
  cps.push(mk2(1.5, 0.0));
  cps.push(mk2(1.5, 0.0));
  cps.push(mk2(1.5, 0.0));
  var bs = curves.b_spline(&cps, 0.5);
  if bs[0] != 1.5 { return 9; }

  // Same-leaf nested sibling must bind its own module.
  var ma = mk_row(1.0, 2.0);
  var mb = mk_row(3.0, 4.0);
  var mh = matrix.hadamard(&ma, &mb);
  if mh[0][0] != 3.0 { return 10; }
  if mh[0][1] != 8.0 { return 11; }

  // Control: unique curve leaf (Vec result, caller read).
  var p0 = mk2(0.0, 0.0);
  var p1 = mk2(1.0, 2.0);
  var p2 = mk2(2.0, 0.0);
  var bq = curves.bezier_quad(&p0, &p1, &p2, 0.5);
  if bq[0] != 1.0 { return 12; }
  if bq[1] != 1.0 { return 13; }
  return 0;
}
