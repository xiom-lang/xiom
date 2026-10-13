// m281 FFI-pointer batch lock (B-24 + B-20 + B-11 shape, 2026-10-13).
// A C extern writes through a caller-owned pointer; the XIOM side reads the
// slot back via raw-pointer indexing (`p[i]`, `slot.as_mut_ptr()[i]`,
// `ffi.ptr_read_*`). Pre-fix every `p[i]` lowered to `xiom_char_at` -- a
// UTF-8 CODEPOINT decoder that clamps with strnlen -- so a binary buffer
// whose byte 0 is 0x00 (little-endian 1024) read back ALL ZEROS (B-24
// ffi_bad reads while byte-indexed Vec reads and C-side peeks saw the
// value; B-20 foreign buffers read zeros). The pointer branch in
// `Expr::Index` now takes the raw-byte load whenever the container is
// XIOM-level pointer-valued instead of falling through to the Str path.
//
// Host: host.c (poke8 / xt_buf_ptr / xt_buf_u32). Copyright (c) 2026
// Eleftherios Notas and The XIOM Authors -- SPDX-License-Identifier:
// MIT OR Apache-2.0
module m281_ffi_ptr_slot_readback

use xiom.io;
use xiom.ffi;

extern "C" {
  fn poke8(dst: *UInt8, v: UInt64);
  fn xt_buf_ptr() -> *UInt8;
  fn xt_buf_u32(i: Int) -> Int32;
}

fn mk_slot(n: Int) -> Vec[UInt8] {
  var v: Vec[UInt8] = Vec[UInt8].new();
  var i = 0;
  while i < n { v.push(0 as UInt8); i = i + 1; }
  return v;
}

// B-24: C writes 8 bytes through the owned slot; read back both through a
// local pointer binding's index AND the direct `as_mut_ptr()[i]` form.
// Value 1024 = bytes 00 04 00 ... -- the exact shape the strnlen clamp ate.
fn round_slot() -> Int {
  var failed = 0;
  var slot = mk_slot(8);
  unsafe { poke8(slot.as_mut_ptr(), 1024 as UInt64); }
  let p = slot.as_mut_ptr();
  let b0 = unsafe { p[0] as Int };
  let b1 = unsafe { p[1] as Int };
  let b2 = unsafe { p[2] as Int };
  let u32 = unsafe { ffi.ptr_read_u32_le(p) };
  let u64 = unsafe { ffi.ptr_read_u64_le(p) };
  if b0 != 0 { failed = failed + 1; }
  if b1 != 4 { failed = failed + 1; }
  if b2 != 0 { failed = failed + 1; }
  if u32 != 1024 { failed = failed + 1; }
  if u64 != 1024 { failed = failed + 1; }
  // direct call-expression index form
  let c1 = unsafe { slot.as_mut_ptr()[1] as Int };
  if c1 != 4 { failed = failed + 1; }
  // byte-indexed Vec read still agrees
  if (slot[1] as Int) != 4 { failed = failed + 1; }
  return failed;
}

// B-20: foreign pointer re-derived as Int + address math, then read.
fn buf_at(off: Int) -> *UInt8
  requires: true
{
  unsafe {
    let base = xt_buf_ptr() as *UInt8;
    return base + off;
  }
}

fn foreign_read() -> Int {
  var failed = 0;
  let p0 = buf_at(0);
  let p1 = buf_at(4);
  let v0 = unsafe { ffi.ptr_read_u32_le(p0) };
  let v1 = unsafe { ffi.ptr_read_u32_le(p1) };
  let c0 = unsafe { xt_buf_u32(0) as Int };
  let c1 = unsafe { xt_buf_u32(1) as Int };
  if v0 != 1065353216 { failed = failed + 1; }
  if v1 != 1073741824 { failed = failed + 1; }
  if v0 != c0 { failed = failed + 1; }
  if v1 != c1 { failed = failed + 1; }
  return failed;
}

fn main() -> Int {
  var failed = 0;
  failed = failed + round_slot();
  failed = failed + foreign_read();
  if failed == 0 { io.println("m281 ptr slot readback ok"); }
  else { io.println("m281 ptr slot readback FAILED=" + to_string(failed)); }
  return failed;
}
