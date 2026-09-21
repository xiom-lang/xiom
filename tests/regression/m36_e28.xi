// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-E28: Deeply nested struct literal
type L1 = { x: Int; }
type L2 = { l1: L1; }
type L3 = { l2: L2; }
type L4 = { l3: L3; }
type L5 = { l4: L4; }
type L6 = { l5: L5; }
type L7 = { l6: L6; }
type L8 = { l7: L7; }
fn main() -> Int {
  var deep = L8{ l7: L7{ l6: L6{ l5: L5{ l4: L4{ l3: L3{ l2: L2{ l1: L1{ x: 42 } } } } } } } };
  if deep.l7.l6.l5.l4.l3.l2.l1.x != 42 { return 1; }
  return 0;
}
