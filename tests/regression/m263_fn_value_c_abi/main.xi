// m263 (REL-20261011-1055 B-13 + REL-20261010-1904, 2026-10-11): fn-VALUE
// C-boundary ABI. Every marshal path must hand C a callable code address:
//   * a fn-typed PARAM relayed straight to an extern fn-typed param
//     (pre-fix: the heap env pointer went to C -> cb saw a data address, 0);
//   * param -> local -> extern (same bug through the copy);
//   * param -> struct field -> extern (the field holds the env verbatim);
//   * `f as Int` of a param/local (field 0, not the env pointer);
//   * Float64 returns through CAST fn-pointers (1904: the i64 fallback read
//     the integer register -- glfwGetTime returned garbage).
// C host: host.c. Exit 0 + "m263 ok" on success.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m263_fn_value_c_abi

use xiom.io;

extern "C" {
  fn xt_cb_int(cb: fn(Int32) -> Int32, x: Int32) -> Int32;
  fn xt_cb_int_fn(cb: fn(Int32) -> Int32, x: Int32) -> Int32;
  fn xt_fp_pi_addr() -> Int;
}

type CbBox = { cb: fn(Int32) -> Int32; }

fn cb_double(x: Int32) -> Int32 {
  return x * 2;
}

// B-13 shape 1: param re-passed to an extern fn-typed param.
fn relay(f: fn(Int32) -> Int32) -> Int
  requires: true
{
  unsafe {
    return xt_cb_int(f, 21) as Int;
  }
}

// B-13 shape 2: param copied to a local, local re-passed to C.
fn relay_copy(f: fn(Int32) -> Int32) -> Int {
  let g: fn(Int32) -> Int32 = f;
  unsafe {
    return xt_cb_int_fn(g, 21) as Int;
  }
}

// B-13 shape 3: param stored in a struct field, field re-passed to C.
fn relay_box(f: fn(Int32) -> Int32) -> Int {
  let b = CbBox { cb: f };
  unsafe {
    return xt_cb_int_fn(b.cb, 21) as Int;
  }
}

// B-13 shape 4: `param as Int` (exit-to-C address).
fn relay_cast(f: fn(Int32) -> Int32) -> Int
  requires: true
{
  unsafe {
    return f as Int;
  }
}

fn main() -> Int {
  unsafe {
    // B-18 (m274): a BARE top-level fn NAME handed to an extern fn-typed
    // param must pass the raw code address C calls -- NOT be wrapped into a
    // closure env. Pre-m274 the wrap handed C a heap env; C called it as
    // code, the unsafe-block trampoline caught the fault and the enclosing
    // function returned 0 early (probe8_name_fnparam.xi).
    if xt_cb_int(cb_double, 21) as Int != 42 {
      return 6;
    }
    if relay(cb_double) != 42 {
      return 1;
    }
    if relay_copy(cb_double) != 42 {
      return 2;
    }
    if relay_box(cb_double) != 42 {
      return 3;
    }
    if relay_cast(cb_double) == 0 {
      return 4;
    }
    // 1904: Float64 return through a CAST fn-pointer (xmm0, not rax).
    let addr = xt_fp_pi_addr();
    let fp = addr as fn() -> Float64;
    if fp() != 3.5 {
      return 5;
    }
  }
  io.println("m263 ok");
  return 0;
}
