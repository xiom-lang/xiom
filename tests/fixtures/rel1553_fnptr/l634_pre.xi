// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// REL-20261010-1553 probe (a): the pre-2026-10-09 L6-34 find_min shape.
// Vec.get returns the element BY VALUE, yet the checker also admits the
// value where fn(&T,&T) expects a reference. The indirect call forwards the
// value and the forwarding thunk inttoptrs it -> 0xC0000005.
module l634_pre.main

use xiom.io;

fn find_min_vec[T](items: Vec[T], less: fn(&T, &T) -> Bool) -> Int
{
  var best: Int = 0;
  var i: Int = 1;
  while i < items.len() {
    if less(items.get(i).unwrap(), items.get(best).unwrap()) {
      best = i;
    }
    i = i + 1;
  }
  return best;
}

fn less_int(a: &Int, b: &Int) -> Bool { return a < b; }

fn main() -> Int
{
  var vi: Vec[Int] = Vec[Int].new();
  vi.push(5);
  vi.push(12);
  vi.push(3);
  let idx = find_min_vec[Int](vi, less_int);
  if idx != 2 {
    io.println("idx=" + idx.to_str());
    return 1;
  }
  io.println("l634 ok");
  return 0;
}
