// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m285 positive control: the MATCHED shape (both arguments are
// &Vec[Vec[Float64]]) must keep compiling and running -- the base-matched
// erasure rule must not reject legal nested-container calls.
// 1x2 * 2x1 product = [[11.0]].
module m285_vec_shape_arg_matched.main

use xiom.math;
use xiom.io;

fn main() -> Int {
  var a = Vec[Vec[Float64]].new();
  var r0 = Vec[Float64].new();
  r0.push(1.0);
  r0.push(2.0);
  a.push(r0);
  var b = Vec[Vec[Float64]].new();
  var c0 = Vec[Float64].new();
  c0.push(3.0);
  var c1 = Vec[Float64].new();
  c1.push(4.0);
  b.push(c0);
  b.push(c1);
  var out = math.matrices.mat_mul(&a, &b);
  if out.len() != 1 { return 1; }
  if out[0].len() != 1 { return 2; }
  if out[0][0] != 11.0 { return 3; }
  io.println("ok");
  return 0;
}
