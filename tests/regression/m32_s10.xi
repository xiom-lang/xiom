// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-S10: Deeply nested structs (4 levels)
type A = { val: Int; }
type B = { a: A; }
type C = { b: B; label: Str; }
type D = { c: C; }
fn main() -> Int {
  var d = D{ c: C{ b: B{ a: A{ val: 42; }; }; label: "nested"; }; };
  if d.c.b.a.val == 42 { return 0; }
  return 1;
}
