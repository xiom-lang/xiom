// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M22: Enum with payload -- match and extract
enum Number { IntVal(v: Int), FloatVal(v: Float64) }
fn extract(n: Number) -> Float64 {
  match n {
    Number.IntVal(v) => v as Float64,
    Number.FloatVal(v) => v,
  }
}
fn main() -> Int {
  var int_case = Number.IntVal(42);
  var float_case = Number.FloatVal(3.14);
  var r1: Float64 = extract(int_case);
  var r2: Float64 = extract(float_case);
  if r1 == 42.0 && r2 == 3.14 {
    return 0;
  }
  return 1;
}
