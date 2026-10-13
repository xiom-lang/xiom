// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m286 lock (REL-20261010-1549-stdlib-7): the stdlib exposure of the
// cross-type generic callback -- `sort.sort_by_key[Int, Str]` must sort
// lexicographically by the Str keys ("1" < "10" < "2"), not by first bytes
// or pointer bits.
module m286_sort_by_key_cross_type.main

use xiom.sort;
use xiom.convert;

fn key_str(x: &Int) -> Str { return convert.int_to_string(*x); }

fn main() -> Int {
  var v: Vec[Int] = Vec[Int].new();
  v.push(2);
  v.push(10);
  v.push(1);
  sort.sort_by_key(&v, key_str);
  if v[0] != 1 { return 1; }
  if v[1] != 10 { return 2; }
  if v[2] != 2 { return 3; }
  return 0;
}
