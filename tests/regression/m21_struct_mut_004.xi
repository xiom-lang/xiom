// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_struct_mut_004
type L1 = { val: Int; }
type L2 = { l1: L1; }
type L3 = { l2: L2; }
fn main() -> Int {
  var x: L3 = L3{ l2: L2{ l1: L1{ val: 1; }; }; };
  x.l2.l1.val = 99;
  if x.l2.l1.val == 99 { return 0; }
  return 1;
}
