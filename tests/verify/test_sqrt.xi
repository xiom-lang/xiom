// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module test_sqrt
fn my_sqrt(x: Float64) -> Float64
  requires: x >= 0.0
  ensures: result >= 0.0
  ensures: result * result <= x + 0.000001
{ return x; }
