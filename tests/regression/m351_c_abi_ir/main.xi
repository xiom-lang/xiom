// m351 (REL-20261016-2031-bindings-3, B-33): IR twin fixture. --emit-ir only;
// asserts that C-boundary signatures are C-lowered on x86_64-pc-windows-msvc:
//   - extern declares/calls use a POINTER for by-value aggregates > 8B;
//   - > 8B returns are `void` with an explicit first sret pointer;
//   - raw `Int as fn(..)` cast calls use the same C-lowered fn-pointer type.
// The pointer-pretend and scalar controls keep their current IR.
//
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module m351_c_abi_ir

type Pair2 = {
  a: Int;
  b: Int;
}

type Pair2i = {
  a: Int32;
  b: Int32;
}

extern "C" {
  fn probe_sum2(p: Pair2, bias: Int) -> Int;
  fn probe_make2(base: Int) -> Pair2;
  fn probe_sum2i(p: Pair2i, bias: Int) -> Int;
  fn probe_make2i(base: Int) -> Pair2i;
  fn probe_mul(x: Int, y: Int) -> Int;
}

pub fn main() -> Int {
  let p = Pair2{ a: 17; b: 29 };
  let pi = Pair2i{ a: 17; b: 29 };
  var direct_sum = 0;
  var direct_pair = Pair2{ a: 0; b: 0 };
  var direct_small = 0;
  var direct_small_pair = Pair2i{ a: 0; b: 0 };
  var scalar = 0;
  unsafe {
    direct_sum = probe_sum2(p, 7);
    direct_pair = probe_make2(500);
    direct_small = probe_sum2i(pi, 7);
    direct_small_pair = probe_make2i(500);
    scalar = probe_mul(6, 7);
  }

  var cast_sum = 0;
  var cast_pair = Pair2{ a: 0; b: 0 };
  var cast_small = 0;
  unsafe {
    let addr = 4096 as Int;
    let f = addr as fn(Pair2, Int) -> Int;
    cast_sum = f(p, 7);
    let g = addr as fn(Int) -> Pair2;
    cast_pair = g(500);
    let h = addr as fn(Pair2i, Int) -> Int;
    cast_small = h(pi, 7);
  }

  return direct_sum + direct_pair.a + direct_small + direct_small_pair.a + scalar + cast_sum + cast_pair.a + cast_small;
}
