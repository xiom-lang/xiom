// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m285 lock (REL-20261010-1549-stdlib-5): passing a flat &Vec[Float64] where
// &Vec[Vec[Float64]] is declared used to compile silently -- the checker's
// structural arg comparison treated the scalar inner arg as an ERASED
// container (one-container leniency with no base match) -- and the callee's
// nested element read then AVed (exit 0xC0000005). It must now be a
// call-site T001 argument mismatch.
module m285_vec_shape_arg_mismatch.main

use xiom.math;

fn main() -> Int {
  var matrix = Vec[Vec[Float64]].new();
  var row = Vec[Float64].new();
  row.push(1.0);
  row.push(2.0);
  matrix.push(row);
  var flat = Vec[Float64].new();
  flat.push(1.0);
  flat.push(2.0);
  // Second argument must be &Vec[Vec[Float64]]; this must NOT compile.
  var out = math.matrices.mat_mul(&matrix, &flat);
  return out.len();
}
