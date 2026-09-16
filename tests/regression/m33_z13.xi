// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-Z13: Compound assignment on struct field -- mutate via field access
type Data = { value: Int; }
fn main() -> Int {
  var d = Data{ value: 10; };
  d.value += 5;
  d.value *= 3;
  d.value -= 10;
  d.value /= 7;
  if d.value == 5 { return 0; }
  return 1;
}
