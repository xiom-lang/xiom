// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-L01: Struct with Int fields -- verify field layout and access
type IntPair = { a: Int; b: Int; c: Int; d: Int; }

fn main() -> Int {
  var s = IntPair{ a: 10; b: 20; c: 30; d: 40; };
  if s.a != 10 { return 1; }
  if s.b != 20 { return 2; }
  if s.c != 30 { return 3; }
  if s.d != 40 { return 4; }
  var sum: Int = s.a + s.b + s.c + s.d;
  if sum != 100 { return 5; }
  return 0;
}
