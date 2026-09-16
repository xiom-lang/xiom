// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-V17: Float return type -- various computation patterns
fn half(x: Float64) -> Float64 { return x / 2.0; }
fn twice(x: Float64) -> Float64 { return x * 2.0; }
fn square(x: Float64) -> Float64 { return x * x; }
fn negate(x: Float64) -> Float64 { return -x; }
fn main() -> Int {
  var a: Float64 = 10.0;
  if half(a) == 5.0 && twice(a) == 20.0 && square(a) == 100.0 && negate(a) == -10.0 {
    return 0;
  }
  return 1;
}
