// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: elif chain with Int8
fn main() -> Int {
  var v: Int8 = -128 as Int8;
  if v > 0 as Int8 { return 1; }
  elif v == 0 as Int8 { return 1; }
  elif v == -128 as Int8 { return 0; }
  else { return 1; }
}
