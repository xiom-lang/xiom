// XVC-C-22 (REL-20261021-1103-xvector) regression fixture part 2 of 2.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// This module name matters: a `xiom.vector.*` struct registers as the bare
// `%struct.Vector`, which is what made the pre-fix `part[j]` copy take the
// 40-byte struct path. Keep the shape in sync with the xvector package's
// src/primitives.xi (Vector = Vec[Float32] + Int).

module xiom.vector.fixture_primitives

pub type Vector = {
  data: Vec[Float32];
  dimension: Int;
}

pub fn Vector.new(dimension: Int) -> Vector {
  var data_vec = Vec[Float32].new();
  var i: Int = 0;
  while i < dimension {
    data_vec.push(0.0);
    i = i + 1;
  }
  return Vector{ data: data_vec, dimension: dimension };
}
