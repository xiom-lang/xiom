// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-N02: Nested struct derive[Eq] -- equality through nested fields
type Inner = { val: Int; } derive[Eq]
type Outer = { inner: Inner; tag: Int; } derive[Eq]
fn main() -> Int {
  var a = Outer{ inner: Inner{ val: 42 }; tag: 1; };
  var b = Outer{ inner: Inner{ val: 42 }; tag: 1; };
  var c = Outer{ inner: Inner{ val: 99 }; tag: 1; };
  if a == b && a != c { return 0; }
  return 1;
}
