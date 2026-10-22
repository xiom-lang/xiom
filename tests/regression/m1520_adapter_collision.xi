// m1520 (REL-20261021-1520-stdlib): adapter terminals must keep the enclosing
// method's generic map (`_max_via[U]` inside MapIter[T,U].max), not emit the
// literal `U` and compare a %struct.Tuple descriptor with i64.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// `MapIter[Int,Int].max()` + `ZipIter[Int,Int].find()` in one program used
// to fail clang with:
//   '%tmp48' defined with type '%struct.Tuple__T__U' but expected 'i64'
// at `icmp sgt i64 %tmp48, %tmp49` in iter._max_via_U.

module m1520_adapter_collision

use xiom.iter;
use xiom.io;
use xiom.convert;

fn dbl(x: Int) -> Int {
  return x * 2;
}

fn first_is_two(p: &(Int, Int)) -> Bool {
  return p.0 == 2;
}

pub fn main() -> Int {
  match iter.range(1, 4).map(dbl).max() {
    Some(v) => {
      io.println("max=" + convert.int_to_string(v));
      if v != 6 { return 1; }
    }
    None => { return 2; }
  }

  match iter.range(1, 5).zip(iter.range(5, 9)).find(first_is_two) {
    Some(p) => {
      io.println("found=" + convert.int_to_string(p.0) + "," + convert.int_to_string(p.1));
      if p.0 != 2 { return 3; }
      if p.1 != 6 { return 4; }
    }
    None => { return 5; }
  }

  io.println("m1520 ok");
  return 0;
}
