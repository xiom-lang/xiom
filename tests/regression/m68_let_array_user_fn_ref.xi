// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// LET-array decision P2 regression (docs/LET_ARRAY_DECISION.md): a user fn's
// `&[N]T` / `&mut [N]T` params must lower to the ELEMENT pointer (the catalog
// generic ABI). Pre-fix they were pointer-to-array (`[3 x i64]*`), so:
//  - a catalog call inside the callee mono'd with T="[3 x i64]" ->
//    `array.len_[3 x i64]_3`, an unquoted LLVM symbol containing `[`/`]`
//    (clang: "expected '(' in call"), and the call arg type disagreed with
//    the mono def's i64* element pointer;
//  - a non-generic `&mut [N]T` element write emitted a two-index GEP on a
//    pointer-to-pointer (clang: "invalid getelementptr indices").
// Probes: tmp/bug_probes/letarr2.xi, letarr2b.xi, letarr2c.xi.
module m68_let_array_user_fn_ref
use xiom.array;

fn sum3(a: &[3]Int) -> Int {
  if array.len(a) != 3 { return -1; }
  return a[0] + a[1] + a[2];
}

fn first_of(a: &[3]Int) -> Int {
  return a[0];
}

fn forward(a: &[3]Int) -> Int {
  return first_of(a);
}

fn bump(a: &mut [3]Int, i: Int, v: Int) {
  a[i] = v;
}

fn narrow_i8(a: &[3]Int8) -> Int {
  return a[0] as Int + a[2] as Int;
}

fn narrow_u8(a: &[3]UInt8) -> Int {
  return a[0] as Int;
}

fn main() -> Int {
  var b = [4, 5, 6];
  if sum3(&b) != 15 { return 1; }
  if first_of(&b) != 4 { return 2; }
  if forward(&b) != 4 { return 3; }
  let c: [3]Int = [7, 8, 9];
  if sum3(&c) != 24 { return 4; }
  bump(&mut b, 1, 50);
  if b[1] != 50 { return 5; }
  if sum3(&b) != 60 { return 6; }
  let n: [3]Int8 = [1 as Int8, 2 as Int8, 3 as Int8];
  if narrow_i8(&n) != 4 { return 7; }
  let u: [3]UInt8 = [200 as UInt8, 1, 2];
  if narrow_u8(&u) != 200 { return 8; }
  return 0;
}
