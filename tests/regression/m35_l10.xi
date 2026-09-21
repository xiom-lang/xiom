// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-L10: Struct alignment check -- verify field layout via struct literal and access
type Aligned = { a: Int; b: Float64; c: Bool; d: Char; }

fn main() -> Int {
  var s = Aligned{ a: 42; b: 3.14; c: true; d: 'Z'; };
  if s.a != 42 { return 1; }
  if s.b > 3.13 && s.b < 3.15 {} else { return 2; }
  if s.c != true { return 3; }
  if s.d != 'Z' { return 4; }
  return 0;
}
