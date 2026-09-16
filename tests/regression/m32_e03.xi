// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-E03: Enum with single-field payload, match extract
enum Number { IntVal(v: Int), FloatVal(v: Float64) }
fn main() -> Int {
  var n = Number.IntVal(42);
  match n {
    IntVal(v) => { if v != 42 { return 1; } }
    FloatVal(_) => { return 1; }
  }
  var f = Number.FloatVal(3.14);
  match f {
    IntVal(_) => { return 1; }
    FloatVal(v) => {
      var diff = v - 3.14;
      if diff < 0.0 { diff = -diff; }
      if diff > 0.001 { return 2; }
    }
  }
  return 0;
}
