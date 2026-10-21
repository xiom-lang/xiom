// XVC-C-22 (REL-20261021-1103-xvector): a Vec-returning call whose FIRST
// argument is a Vec of a 40-byte struct must bind the CALLEE's declared
// element type, not the first argument's.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// `let part = inner(vs, n)` where vs: &Vec[Vector] and inner returns
// Vec[Float32] used to register part's element as Vector (40 bytes). The
// `cents.push(part[j])` copy then used a 40-byte element stride (IR:
// `store i64 40`) instead of 4, corrupting the codebook and eventually the
// guard heap on the second pq_train call. The declared return must win.
// The IR lock (e2e_m_xvc_c22_vec_call_elem_ir) checks that `outer` has no
// `store i64 40`; the runtime check below guards the values.

module m_xvc_c22_vec_call_elem.main

use xiom.io;
use xiom.convert;
use xiom.vector.fixture_primitives;

fn make_vs() -> Vec[Vector] {
  var vs = Vec[Vector].new();
  var v = Vector.new(32);
  vs.push(v);
  return vs;
}

fn inner(vs: &Vec[Vector], n: Int) -> Vec[Float32] {
  var out = Vec[Float32].with_capacity(64);
  var i: Int = 0;
  while i < n {
    out.push(1.5);
    i = i + 1;
  }
  return out;
}

fn outer(vs: &Vec[Vector], n: Int) -> Vec[Float32] {
  var cents = Vec[Float32].with_capacity(64);
  let part = inner(vs, n);
  var j: Int = 0;
  while j < part.len() {
    cents.push(part[j]);
    j = j + 1;
  }
  return cents;
}

pub fn main() -> Int {
  let vs = make_vs();
  let xs = outer(&vs, 2);
  io.println("len=" + convert.int_to_string(xs.len()));
  if xs.len() != 2 { return 1; }
  if xs[0] != 1.5 { return 2; }
  if xs[1] != 1.5 { return 3; }
  io.println("m_xvc_c22 ok");
  return 0;
}
