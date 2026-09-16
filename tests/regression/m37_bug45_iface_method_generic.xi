// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// BUG 45 regression: method-form interface dispatch inside generic-bound
// fns resolved to a stub (contains/is_sorted returned wrong results).
// Both the generic-param receiver (f_eq) and concrete-element receivers
// (contains over a user slice) must dispatch to the impl.
module m37_bug45_iface_method_generic

interface Eq5 {
  fn eq(other: &Self) -> Bool;
}

impl Eq5 for Int {
  fn eq(self, other: &Int) -> Bool {
    var s: Int = self;
    var o: Int = *other;
    return s == o;
  }
}

type Slice5[T] = { data: *T; len: Int; }

fn contains6[T: Eq5](items: &Slice5[Int], value: Int) -> Bool {
  var i: Int = 0;
  while i < items.len {
    var el = items.data[i];
    if el.eq(&value) {
      return true;
    }
    i = i + 1;
  }
  return false;
}

fn f_eq[T: Eq5](x: T, y: T) -> Bool {
  return x.eq(&y);
}

fn main() -> Int {
  var arr = [1, 2, 3, 4, 5];
  var sl = Slice5[Int]{ data: &arr[0]; len: 5; };
  var c1 = contains6(&sl, 3);
  if !c1 { return 1; }
  var c2 = contains6(&sl, 9);
  if c2 { return 2; }
  if !f_eq[Int](3, 3) { return 3; }
  if f_eq[Int](3, 4) { return 4; }
  return 0;
}
