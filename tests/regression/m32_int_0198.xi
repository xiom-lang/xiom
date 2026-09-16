// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Struct with Int16 field
type Pair16 = { a: Int16; b: Int16; }
fn main() -> Int {
  var p: Pair16 = Pair16{ a: 32767, b: 1 };
  var sum: Int16 = p.a + p.b;
  if sum == -32768 as Int16 { return 0; }
  return 1;
}
