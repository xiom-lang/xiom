// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-A11: Multi-dimensional patterns -- nested array indexing simulation
type Triple = { a: Int; b: Int; c: Int; }
fn main() -> Int {
  var r0 = Triple{ a: 1; b: 2; c: 3; };
  var r1 = Triple{ a: 4; b: 5; c: 6; };
  var r2 = Triple{ a: 7; b: 8; c: 9; };
  var sum: Int = r0.a + r0.b + r0.c + r1.a + r1.b + r1.c + r2.a + r2.b + r2.c;
  if sum == 45 { return 0; }
  return 1;
}
