// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// BUG 53 write-facet regression: &mut [N]T param element WRITES -- the
// pointer-typed array param (i64* mono ABI) had no index-write branch
// (the write was silently dropped) and the caller's array-literal
// binding stored the Vec data pointer as the array value.
module m37_bug53_array_ref_write

fn set_first[T, const N: Int](arr: &mut [N]T, v: T) {
  arr[0] = v;
}

fn sum[T, const N: Int](arr: &[N]T) -> Int {
  var i = 0;
  var s = 0;
  while i < N {
    s = s + arr[i];
    i = i + 1;
  }
  return s;
}

fn main() -> Int {
  var a: [3]Int = [10, 20, 30];
  set_first(&mut a, 99);
  if a[0] != 99 { return 1; }
  if a[1] != 20 { return 2; }
  if sum(&a) != 149 { return 3; }
  return 0;
}
