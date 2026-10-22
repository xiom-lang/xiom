// m1550 true tuple shape (REL-20261020-1550-stdlib / REL-20261022-1152-compiler):
// a tuple-RETURNING function value through EnumerateIter[Int].map must bind
// U to the tuple and collect it by value. The outer `.collect()` resolves
// its receiver before the inner `.map()` is emitted, so the receiver's
// instantiation must be inferred on demand; otherwise U collapses to Int and
// `em[1].0` reads 0.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m1550_enumerate_tuple_map_true

use xiom.iter;
use xiom.io;
use xiom.convert;

fn idx_plus(p: (Int, Int)) -> (Int, Int) {
  return (p.0, p.1 + p.0);
}

pub fn main() -> Int {
  var em = iter.range(10, 13).enumerate().map(idx_plus).collect();
  if em.len() != 3 {
    io.println("len=" + convert.int_to_string(em.len()));
    return 1;
  }
  if em[0].0 != 0 { io.println("i0=" + convert.int_to_string(em[0].0)); return 2; }
  if em[1].0 != 1 { io.println("i1=" + convert.int_to_string(em[1].0)); return 3; }
  if em[2].0 != 2 { io.println("i2=" + convert.int_to_string(em[2].0)); return 4; }
  if em[0].1 != 10 { io.println("v0=" + convert.int_to_string(em[0].1)); return 5; }
  if em[1].1 != 12 { io.println("v1=" + convert.int_to_string(em[1].1)); return 6; }
  if em[2].1 != 14 { io.println("v2=" + convert.int_to_string(em[2].1)); return 7; }
  io.println("m1550 true ok");
  return 0;
}
