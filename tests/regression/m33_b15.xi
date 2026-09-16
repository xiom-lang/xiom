// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-B15: Immutable let vs mutable var -- let is immutable, var is mutable
fn main() -> Int {
  let a = 10;
  var b = 5;
  b = b + a;
  if a == 10 && b == 15 { return 0; }
  return 1;
}
