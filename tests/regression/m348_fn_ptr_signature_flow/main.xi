// m348 (REL-20261016-2002-packages, 2026-10-16): fn-pointer RETURN-ABI
// signature flow. A fn pointer bound from a CALL whose DECLARED return is
// fn(..) (`extern fn get_cos() -> fn(Float64) -> Float64;` -- the natural
// C-typedef getter / dlsym-loader shape) must record the callee's return
// signature at the binding site. Pre-fix no record was written, the call
// site fell back to i64 (`call i64 %p` + sitofp in the IR while the callee
// returns in xmm0) and Float64/Str returns surfaced as garbage
// (cos(0.0) via pointer = 0, add = 16, copies = pointer-decimal class).
// Same class: a plain Ident COPY of a raw fn local (`let b = a;`) lost the
// record because the copy branch sat under the env-provenance branch.
// C host: host.c. Exit 0 + "m348 ok" on success.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m348_fn_ptr_signature_flow

use xiom.io;

extern "C" {
  fn mt348_get_cos_fn() -> fn(Float64) -> Float64;
  fn mt348_get_add_fn() -> fn(Float64, Float64) -> Float64;
  fn mt348_get_label_fn() -> fn() -> Str;
  fn mt348_cos_addr() -> Int;
  fn mt348_cos_direct(x: Float64) -> Float64;
}

fn i_of(v: Float64) -> Int {
  return v as Int;
}

pub fn main() -> Int {
  var fail = 0;

  // E1: call-valued binding, NO annotation -- the natural loader shape.
  // Pre-fix: cos(0.0) read rax garbage -> 0.0.
  let f1 = unsafe { mt348_get_cos_fn() };
  let v1 = f1(0.0);
  io.println("E1_cos_int=" + i_of(v1).to_str());
  if v1 != 1.0 { fail = fail + 1; }

  // E2 (control, recorded annotated path): must keep working.
  let f2: fn(Float64) -> Float64 = unsafe { mt348_get_cos_fn() };
  let v2 = f2(0.0);
  io.println("E2_cos_int=" + i_of(v2).to_str());
  if v2 != 1.0 { fail = fail + 2; }

  // E3: two-arg typed getter called with literals.
  // Pre-fix: add(1.5, 2.25) read rax -> 16.
  let f3 = unsafe { mt348_get_add_fn() };
  let v3 = f3(1.5, 2.25);
  io.println("E3_add_int=" + i_of(v3).to_str());
  if v3 != 3.75 { fail = fail + 4; }

  // F1: Ident copy of the ANNOTATED (recorded) local -- the record must
  // survive a plain copy. Pre-fix rc 1 (copy lost the record).
  let fb = f2;
  let vb = fb(0.0);
  io.println("F1_copy_annotated_int=" + i_of(vb).to_str());
  if vb != 1.0 { fail = fail + 8; }

  // F2: Ident copy of the unannotated call-valued local.
  let fc = f3;
  let vc = fc(1.5, 2.25);
  io.println("F2_copy_callval_int=" + i_of(vc).to_str());
  if vc != 3.75 { fail = fail + 16; }

  // E5: Str-return typed getter (pointer class through the pointer).
  // Pre-fix printed the pointer as a decimal.
  let fl = unsafe { mt348_get_label_fn() };
  let sl = fl();
  io.println("E5_label=" + sl);
  if sl != "label-ok" { fail = fail + 32; }

  // F3: copy of the Str-return local.
  let fm = fl;
  let sm = fm();
  io.println("F3_copy_label=" + sm);
  if sm != "label-ok" { fail = fail + 64; }

  // F6 (control, recorded path): Int address -> `as fn` cast.
  unsafe {
    let addr = mt348_cos_addr();
    let f6 = addr as fn(Float64) -> Float64;
    let v6 = f6(0.0);
    io.println("F6_int_addr_cast_int=" + i_of(v6).to_str());
    if v6 != 1.0 { fail = fail + 128; }
  }

  // E6 (control): direct extern call of the same function.
  let vd = unsafe { mt348_cos_direct(0.0) };
  if vd != 1.0 { fail = fail + 256; }

  if fail != 0 {
    io.println("m348 fail_bits=" + fail.to_str());
    return fail;
  }
  io.println("m348 ok");
  return 0;
}
