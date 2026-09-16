// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// BUG 46 regression: reading a field of a USER-DEFINED generic struct
// through a `&T` param returned garbage (the mono param degraded to i64*).
// By-value params, &Box2[T] field reads, and &Slice[T] len reads must all
// work inside generic fns.
module m37_bug46_generic_struct_ref

type Box2[T] = { v: T; }
type Slice9[T] = { data: *T; len: Int; }

fn get_v[T](b: &Box2[T]) -> T { return b.v; }
fn get_v_byval[T](b: Box2[T]) -> T { return b.v; }
fn len_of[T](s: &Slice9[T]) -> Int { return s.len; }
fn first_is[T](items: &Slice9[T], value: T) -> Bool {
  return items.data[0] == value;
}

fn main() -> Int {
  var b = Box2[Int]{ v: 42; };
  var byval = get_v_byval(b);
  if byval != 42 { return 1; }
  var via_ref = get_v(&b);
  if via_ref != 42 { return 2; }
  var arr = [1, 2, 3, 4, 5];
  var sl = Slice9[Int]{ data: &arr[0]; len: 5; };
  var l = len_of(&sl);
  if l != 5 { return 3; }
  if !first_is[Int](&sl, 1) { return 4; }
  if first_is[Int](&sl, 9) { return 5; }
  return 0;
}
