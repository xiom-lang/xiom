// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module test_max
fn my_max(a: Int, b: Int) -> Int
  ensures: result >= a
  ensures: result >= b
  ensures: result == a || result == b
{ if a > b { return a; } return b; }
