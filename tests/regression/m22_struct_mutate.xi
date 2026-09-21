// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M22: Struct field mutation and nested access
type Inner = { val: Int; }
type Outer = { name: Str; inner: Inner; }
fn main() -> Int {
  var i = Inner{ val: 10; };
  var o = Outer{ name: "test"; inner: i; };
  o.inner.val = 99;
  return o.inner.val;
}
