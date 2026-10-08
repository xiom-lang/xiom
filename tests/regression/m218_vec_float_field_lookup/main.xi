// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m218 (XVC-C-05): `a.data[i] * b.data[i]` on a `Vec[Float32]` struct field
// must lower as float math. Pre-fix `vec_elem_float_type`'s Field arm scanned
// type_meta (HashMap order) and broke on the first key ending with the base
// type -- the generated aggregate `Option__Vector` (no `data` field) could
// shadow `Vector`, the read fell to the scalar elem_load and the f32 BIT
// PATTERNS were multiplied as i64 + sitofp'd (~5/6 compiles, RandomState).
// No module header on purpose: the engine lane registers `Vector` bare, and
// a file-level module header would prefix the key and hide the collision.

type Vector = {
  data: Vec[Float32];
  dimension: Int;
}

fn mk_vec(a: Float32, b: Float32) -> Vector {
  var d: Vec[Float32] = Vec[Float32].new();
  d.push(a);
  d.push(b);
  return Vector { data: d, dimension: 2 };
}

// Forces the generated Option__Vector aggregate into type_meta so the
// pre-fix suffix scan had something to shadow `Vector` with.
fn maybe(v: Vector) -> Option[Vector] {
  return Some(v);
}

fn dot(a: &Vector, b: &Vector) -> Float32 {
  var sum: Float32 = 0.0;
  var i: Int = 0;
  while i < a.dimension {
    sum = sum + a.data[i] * b.data[i];
    i = i + 1;
  }
  return sum;
}

fn f32_approx(a: Float32, b: Float32, eps: Float32) -> Bool {
  var diff = a - b;
  if diff < 0.0 { diff = -diff; }
  return diff < eps;
}

fn main() -> Int {
  let a = mk_vec(0.9, 1.0);
  let b = mk_vec(1.0, 2.0);
  let c = mk_vec(3.0, 4.0);
  let m = maybe(c);
  if m.is_none { return 2; }
  let d = dot(&a, &b);
  if !f32_approx(d, 2.9, 0.001) { return 1; }
  return 0;
}
