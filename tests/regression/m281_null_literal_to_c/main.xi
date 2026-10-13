// m281 B-23 lock (null literal -> C, 2026-10-13). The bare `null` pointer
// literal must cross to a C extern as a true NULL pointer. Pre-fix the
// Ident arg took the local-type arm (no binding for `null`), classified as
// NOT pointer-valued, and the materialization path emitted
// `alloca i8 + trunc/store` -- C received the ADDRESS OF A STACK BYTE
// (varying per run: dx12 CreateCommittedResource(pOptimizedClearValue=null)
// -> E_INVALIDARG). A zero pointer that ORIGINATED from C converts
// correctly on both builds; the lock asserts all four paths are 0.
// Host: host.c (xs_ptr_val / xs_null). Copyright (c) 2026 Eleftherios
// Notas and The XIOM Authors -- SPDX-License-Identifier: MIT OR Apache-2.0
module m281_null_literal_to_c

use xiom.io;

extern "C" {
  fn xs_ptr_val(p: *UInt8) -> Int;
  fn xs_null() -> *UInt8;
}

fn get_null() -> *UInt8
  requires: true
{
  unsafe { return xs_null(); }
}

fn forward(p: *UInt8) -> Int
  requires: true
{
  unsafe { return xs_ptr_val(p); }
}

fn probe_direct() -> Int
  requires: true
{
  unsafe { return xs_ptr_val(null); }
}

fn main() -> Int {
  let direct = probe_direct();
  let via_fn = forward(null);
  let c_null = get_null();
  let from_c = unsafe { xs_ptr_val(c_null) as Int };
  let via_var = unsafe { xs_ptr_val(c_null as *UInt8) as Int };
  io.println("m281 null: direct=" + to_string(direct) + " via_fn=" + to_string(via_fn)
    + " from_c=" + to_string(from_c) + " via_var=" + to_string(via_var));
  var failed = 0;
  if direct != 0 { failed = failed + 1; }
  if via_fn != 0 { failed = failed + 1; }
  if from_c != 0 { failed = failed + 1; }
  if via_var != 0 { failed = failed + 1; }
  return failed;
}
