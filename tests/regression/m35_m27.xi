// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-M27: round Float64 to nearest Int (ties toward positive infinity)
fn round_to_int(x: Float64) -> Int {
  if x >= 0.0 { return (x + 0.5) as Int; }
  return (x - 0.5) as Int;
}
fn main() -> Int {
  if round_to_int(3.2) == 3 && round_to_int(3.7) == 4 && round_to_int(-2.3) == -2 && round_to_int(-2.8) == -3 && round_to_int(0.0) == 0 { return 0; }
  return 1;
}
