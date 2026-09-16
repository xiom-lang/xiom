// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_struct_mut_020
type Stats = { min: Int; max: Int; avg: Int; }
fn main() -> Int {
  var s: Stats = Stats{ min: 0; max: 0; avg: 0; };
  s.min = 5;
  s.max = 95;
  s.avg = 50;
  if s.min == 5 && s.max == 95 && s.avg == 50 { return 0; }
  return 1;
}
