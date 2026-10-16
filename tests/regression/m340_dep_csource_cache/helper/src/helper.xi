// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m340_cache_helper.helper

extern "C" {
  fn m340_cache_a(x: Int) -> Int;
  fn m340_cache_b(x: Int) -> Int;
  fn m340_cache_c(x: Int) -> Int;
}

pub fn sum(x: Int) -> Int {
  let a = unsafe { m340_cache_a(x) };
  let b = unsafe { m340_cache_b(x) };
  let c = unsafe { m340_cache_c(x) };
  return a + b + c;
}
