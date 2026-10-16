// m348 (REL-20261016-2002-packages, 2026-10-16): pure-XIOM companion of the
// C typed-getter lock (m348_fn_ptr_signature_flow, host.c driven). A fn
// pointer bound from a CALL whose DECLARED return is fn(..) --
// `let f = make_fit();` where `fn make_fit() -> fn(Float64) -> Float64` --
// must record the callee's return signature at the binding; pre-fix the
// call site fell back to i64 (`call i64` while the callee returns xmm0) and
// the Float64 result surfaced as raw-bits garbage (4613937818241073152 for
// 3.0). A plain Ident copy (`let g = f;`) must keep the record too.
// Pre-fix: P1 garbage, P2 wrong, rc 3. Post-fix: rc 0 + "m348 pure ok".
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m348_fn_ptr_signature_flow_pure

use xiom.io;

fn fit(v: Float64) -> Float64 {
  return v * 2.0;
}

fn make_fit() -> fn(Float64) -> Float64 {
  return fit;
}

pub fn main() -> Int {
  var fail = 0;

  // P1: unannotated call-valued binding of a fn-returning internal fn.
  let f = make_fit();
  let v = f(1.5);
  io.println("P1_fit_int=" + (v as Int).to_str());
  if v != 3.0 { fail = fail + 1; }

  // P2: Ident copy keeps the recorded return class.
  let g = f;
  let v2 = g(2.5);
  io.println("P2_copy_fit_int=" + (v2 as Int).to_str());
  if v2 != 5.0 { fail = fail + 2; }

  if fail != 0 {
    io.println("m348 pure fail_bits=" + fail.to_str());
    return fail;
  }
  io.println("m348 pure ok");
  return 0;
}
