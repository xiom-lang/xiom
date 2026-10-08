// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m222 (XVC-C-07): `Str::from_utf8(&Vec[UInt8])` type-checked but emitted
// invalid LLVM IR (clang: "invalid getelementptr indices"): the reference
// argument arrives as `%struct.Vec*` and the intrinsic path GEP'd the fields
// through the pointer value as if it were the struct. References must read the
// fields through the pointee; the by-value form and `from_bytes` keep working.

module m222_from_utf8_vec_ref

use xiom.string.slice;

fn by_ref_ok() -> Bool {
  var kb = Vec[UInt8].new();
  kb.push(65u8);   // 'A'
  kb.push(66u8);   // 'B'
  var s = Str::from_utf8(&kb);
  return s == "AB";
}

fn by_value_ok() -> Bool {
  var kb = Vec[UInt8].new();
  kb.push(65u8);
  kb.push(66u8);
  var t = Str::from_utf8(kb);
  return t == "AB";
}

fn from_bytes_ref_ok() -> Bool {
  var kb = Vec[UInt8].new();
  kb.push(67u8);   // 'C'
  var b = Str::from_bytes(&kb);
  return b == "C";
}

fn main() -> Int {
  if !by_ref_ok() { return 1; }
  if !by_value_ok() { return 2; }
  if !from_bytes_ref_ok() { return 3; }
  return 0;
}
