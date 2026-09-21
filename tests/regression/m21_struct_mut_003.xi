// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_struct_mut_003
type Inner = { val: Int; }
type Outer = { inner: Inner; tag: Int; }
fn main() -> Int {
  var o: Outer = Outer{ inner: Inner{ val: 5; }; tag: 10; };
  o.inner.val = 99;
  if o.inner.val == 99 && o.tag == 10 { return 0; }
  return 1;
}
