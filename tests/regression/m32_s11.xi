// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-S11: Struct mutation -- mutate nested fields through arithmetic
type Inner = { val: Int; scale: Float64; }
type Outer = { inner: Inner; id: Int; }
fn main() -> Int {
  var o = Outer{ inner: Inner{ val: 10; scale: 2.0; }; id: 7; };
  o.inner.val = 99;
  o.inner.val = o.inner.val + 1;
  o.id = o.inner.val;
  if o.inner.val == 100 && o.id == 100 { return 0; }
  return 1;
}
