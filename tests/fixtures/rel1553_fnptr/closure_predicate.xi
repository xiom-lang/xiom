// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// REL-20261010-1553 probe (c): a closure-literal predicate passed to a
// `fn(&T,&T) -> Bool` parameter. Pre-fix find_min_int silently returns 0
// because the comparison operates on pointer bits instead of the referents.
module closure_predicate.main

use xiom.io;

fn find_min_int(vi: &Vec[Int], less: fn(&Int, &Int) -> Bool) -> Int
{
  var best: Int = 0;
  var i: Int = 1;
  while i < vi.len() {
    if less(&vi[i], &vi[best]) { best = i; }
    i = i + 1;
  }
  return best;
}

fn main() -> Int
{
  var vi: Vec[Int] = Vec[Int].new();
  vi.push(5);
  vi.push(12);
  vi.push(3);
  let idx = find_min_int(&vi, |a, b| a < b);
  if idx != 2 {
    io.println("idx=" + idx.to_str());
    return 1;
  }
  io.println("closure predicate ok");
  return 0;
}
