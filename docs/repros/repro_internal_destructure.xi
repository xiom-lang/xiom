// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module repro_internal_destructure

fn helper() -> (Vec[UInt8], Int) {
  var v = Vec[UInt8].new();
  v.push(1); v.push(2); v.push(3);
  return (v, 14);
}

pub fn roundtrip() -> Result[(Vec[UInt8], Vec[UInt8]), Str] {
  // INTERNAL tuple destructure, then build a DIFFERENT tuple of Vecs
  let (expanded, nr) = helper();
  var out1 = Vec[UInt8].new();
  out1.push(expanded[0]);
  out1.push(expanded[1]);
  var out2 = Vec[UInt8].new();
  var i = 0;
  while i < nr {
    out2.push(i as UInt8);
    i = i + 1;
  }
  return Ok((out1, out2));
}
