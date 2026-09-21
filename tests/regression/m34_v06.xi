// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-V06: Float multiplication precision (commutativity, identity, zero)
fn main() -> Int {
  var a: Float64 = 3.0;
  var b: Float64 = 7.0;
  var prod1: Float64 = a * b;
  var prod2: Float64 = b * a;
  var ident: Float64 = a * 1.0;
  var zero: Float64 = a * 0.0;
  if prod1 == prod2 && ident == a && zero == 0.0 {
    return 0;
  }
  return 1;
}
