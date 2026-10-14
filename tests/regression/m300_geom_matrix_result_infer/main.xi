// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m300 lock (REL-20261010-1549-stdlib-3): call-site type inference for
// xiom.geom.matrix results of type Vec[Vec[Float64]] lost a nesting level
// when the local was declared WITHOUT an explicit type (`var z = ...;
// z[0].len()` read 0) and for tuple elements (`var l2 = lu.0;`). Root
// cause (codegen): `type_string_full` fell back to `type_from_ast` for
// tuple elements, erasing container args -- `lu_decompose ->
// (Vec[Vec[Float64]], Vec[Vec[Float64]])` recorded "Tuple__Vec__Vec", so
// `lu.0` resolved to a bare "Vec" and the row read went through the scalar
// elem path. Mirrors the archived witness p_geom_matrix_result_infer.xi in
// the stdlib known_failures corpus.
module m300_geom_matrix_result_infer.main

use xiom.geom.matrix;
use xiom.geom.mat;

fn main() -> Int {
  // WAS BROKEN: inferred nested result loses a level (row len 0 pre-fix).
  var z1 = matrix.zero(2, 2);
  if z1[0].len() != 2 { return 1; }
  // Correct: explicit annotation.
  var z2: Vec[Vec[Float64]] = matrix.zero(2, 2);
  if z2[0].len() != 2 { return 2; }
  // Control: the same signature via xiom.geom.mat is correct un-annotated.
  var c = mat.mat_identity(2);
  if c[0].len() != 2 { return 3; }
  // WAS BROKEN: tuple element without annotation (identity keeps LU non-empty).
  var id: Vec[Vec[Float64]] = matrix.identity(2);
  var lu = matrix.lu_decompose(&id);
  var l2 = lu.0;
  if l2[0].len() != 2 { return 4; }
  // Correct: tuple element with annotation.
  var l: Vec[Vec[Float64]] = lu.0;
  if l[0].len() != 2 { return 5; }
  return 0;
}
