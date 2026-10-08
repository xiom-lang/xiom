// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m224 (C-ORBIT-03): a FIELD receiver of a pointer-self method must pass the
// field's ADDRESS, not a copy. `o.inner.bump()` (base = `&mut Outer` param) and
// `c.inner.bump()` (base = local struct) used to alloca a temp copy, bump it and
// discard the write (probe A=0/C=0; StorageEngine.cache_page lost the page).
// Nested `h.mid.inner.bump()` follows the same Ref-machinery recursion.

module m224_nested_field_mut_receiver

type Inner = {
  n: Int;
}

pub fn Inner.bump(x: &mut Inner) {
  x.n = x.n + 1;
}

type Outer = {
  inner: Inner;
}

pub fn Outer.bump(o: &mut Outer) {
  o.inner.bump();
}

type Holder = {
  mid: Outer;
}

pub fn Holder.deep(h: &mut Holder) {
  h.mid.inner.bump();
}

fn main() -> Int {
  // A: nested receiver on a reference-param base.
  var a = Outer{ inner: Inner{ n: 0 } };
  Outer.bump(&mut a);
  if a.inner.n != 1 { return 1; }

  // C: nested receiver on a local struct base.
  var c = Outer{ inner: Inner{ n: 0 } };
  c.inner.bump();
  if c.inner.n != 1 { return 2; }

  // Deep: double-nested field receiver.
  var h = Holder{ mid: Outer{ inner: Inner{ n: 0 } } };
  h.deep();
  if h.mid.inner.n != 1 { return 3; }

  return 0;
}
