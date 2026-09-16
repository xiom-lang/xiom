// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module repro_tuple_destructure

pub fn key_expansion() -> (Vec[UInt8], Int) {
  var v = Vec[UInt8].new();
  v.push(1); v.push(2); v.push(3);
  return (v, 14);
}

pub fn use_key(v: &Vec[UInt8], nr: Int) -> Int {
  if v.len() != 3 { return 1; }
  if v[2] != 3 { return 2; }
  if nr != 14 { return 3; }
  return 0;
}
