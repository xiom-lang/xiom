// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Struct field access with nested struct
type Inner = { a: Int8; b: UInt16; }
type Outer = { inner: Inner; tag: UInt8; }
fn main() -> Int {
  var o: Outer = Outer{ inner: Inner{ a: 10 as Int8; b: 20 as UInt16; }; tag: 30 as UInt8; };
  var casted: UInt16 = o.inner.b + o.tag;
  if casted == 50 as UInt16 { return 0; }
  return 1;
}
