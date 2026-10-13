// m279 (REL-20261011-1512-bindings-2 B-17 / REL-20261011-1437-bindings-3
// B-16): raw-pointer arithmetic on an owned buffer pointer must yield
// base+N and ffi.ptr_write_* at base+N must land. Pre-fix `base + N` was
// lowered as Str concatenation (a fresh concat buffer: wild addresses) so
// the +4/+8 writes went to unowned memory. Returns 0 only when every
// delta is exact and both slot sums are 66.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module m279_ffi_ptr_arith;

use xiom.io;
use xiom.convert;
use xiom.ffi;

extern "C" {
  fn xt_sum3(p: *UInt8) -> Int32;
}

fn slot(n: Int) -> Vec[UInt8] {
  var s: Vec[UInt8] = Vec[UInt8].new();
  var i: Int = 0;
  while i < n {
    s.push(0 as UInt8);
    i = i + 1;
  }
  return s;
}

fn wr32(buf: &mut Vec[UInt8], off: Int, v: Int)
  requires: buf.len() >= off + 4
{
  buf[off + 0] = ( v        & 0xFF) as UInt8;
  buf[off + 1] = ((v >>  8) & 0xFF) as UInt8;
  buf[off + 2] = ((v >> 16) & 0xFF) as UInt8;
  buf[off + 3] = ((v >> 24) & 0xFF) as UInt8;
}

fn main() -> Int {
  var fails: Int = 0;
  unsafe {
    var slot1 = slot(32);
    let base = slot1.as_mut_ptr();
    let ib = base as Int;
    if ((base + 0) as Int) - ib != 0 { fails = fails + 1; }
    if ((base + 1) as Int) - ib != 1 { fails = fails + 1; }
    if ((base + 4) as Int) - ib != 4 { fails = fails + 1; }
    if ((base + 12) as Int) - ib != 12 { fails = fails + 1; }
    if ((base + 100) as Int) - ib != 100 { fails = fails + 1; }
    if (base as Int + 12) - ib != 12 { fails = fails + 1; }
    var slot2 = slot(32);
    let base2 = slot2.as_mut_ptr();
    let ib2 = base2 as Int;
    if ((base2 + 12) as Int) - ib2 != 12 { fails = fails + 1; }

    // B-16: ffi.ptr_write at base+N (call-argument position).
    var a = slot(12);
    ffi.ptr_write_u32_le(a.as_mut_ptr(), 11);
    ffi.ptr_write_u32_le(a.as_mut_ptr() + 4, 22);
    ffi.ptr_write_u32_le(a.as_mut_ptr() + 8, 33);
    let ra = xt_sum3(a.as_mut_ptr()) as Int;
    if ra != 66 { fails = fails + 1; }

    var b = slot(12);
    wr32(&mut b, 0, 11);
    wr32(&mut b, 4, 22);
    wr32(&mut b, 8, 33);
    let rb = xt_sum3(b.as_mut_ptr()) as Int;
    if rb != 66 { fails = fails + 1; }
    io.println("m279_ffi_ptr_arith: ptr_write_sum=" + int_to_string(ra) + " byte_index_sum=" + int_to_string(rb) + " fails=" + int_to_string(fails));
  }
  return fails;
}
