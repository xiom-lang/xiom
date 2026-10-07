// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m200 (stdlib known_failures p_rvalue_float_vec_index): indexing the RVALUE
// of a Vec[Float64]-returning call must read the stored double, like the
// bound-local path. Pre-fix the scalar elem_load yielded raw i64 bits and the
// comparison sitofp'd the bit pattern (1.0 -> 4.6e18) -> rc 1.
fn mk_f() -> Vec[Float64] {
  var v = Vec[Float64].new();
  v.push(1.0);
  return v;
}

fn mk_i() -> Vec[Int] {
  var v = Vec[Int].new();
  v.push(5);
  return v;
}

fn main() -> Int {
  // Int control: the same rvalue read was always correct.
  if mk_i()[0] != 5 { return 2; }
  // Bound Float64 locals are correct (the wave-77 workaround).
  var vf = mk_f();
  if vf[0] != 1.0 { return 3; }
  let lf = mk_f();
  if lf[0] != 1.0 { return 4; }
  // The bug: inline indexing of the returned Vec[Float64] rvalue misread.
  if mk_f()[0] != 1.0 { return 1; }
  return 0;
}
