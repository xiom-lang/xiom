// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m349 lock (REL-20261016-2004-packages / XARROW-ARRAYNAME): catalog-side
// module declaring a struct named `Array` -- the arrow `XARROW-ARRAYNAME`
// shape (an imported C struct sharing a name with a builtin). The declaration
// must inject like any struct: by-value return, &ref param, a registered
// method, a nested struct field and a Vec element are all exercised from
// main.xi against exact runtime values.
module xiom.m349x.def;

pub type Array = {
  n: Int;
  m: Int;
}

pub fn mk_arr(n: Int) -> Array {
  return Array { n: n, m: n * 2 };
}

pub fn arr_sum(a: &Array) -> Int {
  return a.n + a.m;
}

pub fn Array.span(a: &Array) -> Int {
  return a.n + a.m;
}

pub type Holder = {
  arr: Array;
  tag: Int;
}

pub fn mk_holder(n: Int) -> Holder {
  return Holder { arr: Array { n: n, m: n + 1 }, tag: n * 3 };
}

pub fn holder_sum(h: &Holder) -> Int {
  return h.arr.n + h.arr.m + h.tag;
}

pub fn mk_arr_vec(n: Int) -> Vec[Array] {
  var v = Vec[Array].new();
  v.push(Array { n: n, m: n * 2 });
  return v;
}
