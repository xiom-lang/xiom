// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m251 (sortbykey tail): Str relational ordering must be lexicographic
// (strcmp), not pointer/byte order. `core.Str.compare` used to load only
// the FIRST BYTE of each operand (the i8* auto-deref path), so "1" vs
// "10" reported equal and sort_by_key_Int_Str silently mis-sorted. Also
// covers an i64-held Str-substituted generic key.
module m251_str_ordering

use xiom.sort;
use xiom.convert;

fn key_str(x: &Int) -> Str { return convert.int_to_string(*x); }

fn main() -> Int {
  // Direct literal ordering.
  if !("a" < "b") { return 1; }
  if "b" < "a" { return 2; }
  if !("a" <= "a") { return 3; }
  if !("b" > "a") { return 4; }
  if !("b" >= "b") { return 5; }
  // Prefix must sort before its extension.
  if !("1" < "10") { return 6; }

  // The stdlib exposure: T=Int, K=Str insertion sort.
  var v = Vec[Int].new();
  v.push(2); v.push(10); v.push(1);
  sort.sort_by_key(&v, key_str);
  if v[0] != 1 { return 7; }
  if v[1] != 10 { return 8; }
  if v[2] != 2 { return 9; }

  return 0;
}
