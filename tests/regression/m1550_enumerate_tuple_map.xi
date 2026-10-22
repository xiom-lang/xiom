// m1550 (REL-20261020-1550-stdlib): a tuple-returning function value through
// EnumerateIter.map must bind the tuple by value, not as a boxed pointer.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Requires stdlib >= the EnumerateIter[Int].map specialization
// (stdlib d53f93a / stdlib-v0.64.9). With the generic EnumerateIter[T].map
// the declared return MapIter[(Int, T), U] leaked the placeholder T, so
// collect/next specialized with Int and self.f(v) passed the boxed Option
// tuple pointer; em held pointer-like values.

module m1550_enumerate_tuple_map

use xiom.iter;
use xiom.io;
use xiom.convert;

fn idx_plus(p: (Int, Int)) -> Int {
  return p.0 + p.1;
}

pub fn main() -> Int {
  var em = iter.range(10, 13).enumerate().map(idx_plus).collect();
  io.println("len=" + convert.int_to_string(em.len()));
  if em.len() != 3 { return 1; }
  var i: Int = 0;
  while i < em.len() {
    io.println("em[" + convert.int_to_string(i) + "]=" + convert.int_to_string(em[i]));
    i = i + 1;
  }
  if em[0] != 10 { return 2; }
  if em[1] != 12 { return 3; }
  if em[2] != 14 { return 4; }
  io.println("m1550 ok");
  return 0;
}
