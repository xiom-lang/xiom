// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Integer in struct with mixed int types
type Data = { a: Int8; b: Int16; c: Int32; }
fn main() -> Int {
  var d = Data{ a: 10; b: 20; c: 30; };
  var total: Int = d.a as Int + d.b as Int + d.c as Int;
  if total == 60 {
    return 0;
  }
  return 1;
}
