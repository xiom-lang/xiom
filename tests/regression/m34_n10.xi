// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-N10: Struct with multiple derives (Eq+Clone) -- test both operations
type Item = { id: Int; value: Int; } derive[Eq, Clone]
fn main() -> Int {
  var a = Item{ id: 1; value: 100; };
  var b = a.clone();
  if a == b && b.value == 100 { return 0; }
  return 1;
}
