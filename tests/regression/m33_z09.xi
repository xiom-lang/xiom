// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-Z09: Float64 /= -- basic compound divide
fn main() -> Int {
  var f: Float64 = 15.0;
  f /= 3.0;
  if f == 5.0 { return 0; }
  return 1;
}
