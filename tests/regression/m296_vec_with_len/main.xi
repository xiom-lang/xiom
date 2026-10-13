// m296 (REL-20261011-2320-bindings #1 HIGH): Vec[UInt8].with_len(n) is a
// sized buffer under the documented Vec contract of 8-byte slots -- the
// stdlib zero-fill writes n * 8 bytes. Pre-fix every construction site
// allocated cap * elem_size (Vec[UInt8]: 16 x 1 = 16 bytes), so the
// with_len(16) fill wrote 128 bytes into a 16-byte region (ASAN: WRITE of
// size 128 past a 16-byte region allocated by Vec.with_len_UInt8; the
// STATUS_HEAP_CORRUPTION flakiness class seen by the bindings lane).
// Construction buffers now floor the slot at 8 bytes (vec_slot_size);
// boundary writes across the full n * 8 slot area gate the exit code.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module m296_vec_with_len;

use xiom.io;
use xiom.convert;

fn main() -> Int {
  var fails: Int = 0;

  // A: Vec[UInt8].with_len(16) -- boundary writes across all 16 elements.
  var a = Vec[UInt8].with_len(16);
  if a.len() != 16 {
    fails = fails + 1;
    io.println("  [BROKEN] A with_len(16) len != 16");
  }
  var i = 0;
  while i < 16 {
    a[i] = (i + 1) as UInt8;
    i = i + 1;
  }
  var sum = 0;
  i = 0;
  while i < 16 {
    sum = sum + (a[i] as Int);
    i = i + 1;
  }
  if sum != 136 {
    fails = fails + 1;
    io.println("  [CORRUPT] A byte readback sum=" + convert.int_to_string(sum));
  }

  // A2: the documented 8-byte slot area is addressable to its end (the FFI
  // out-param shape): write/read the LAST slot (bytes 120..127 of 128).
  unsafe {
    var p = a.data as *UInt64;
    p[15] = 72057594037927935 as UInt64;
    if (p[15] as Int) != 72057594037927935 {
      fails = fails + 1;
      io.println("  [CORRUPT] A2 last 8-byte slot readback");
    }
  }

  // B: Vec[UInt8].with_len(3) -- the smallest pre-fix overflowing shape
  // (24-byte zero-fill into a 16-byte region).
  var b = Vec[UInt8].with_len(3);
  if b.len() != 3 {
    fails = fails + 1;
    io.println("  [BROKEN] B with_len(3) len != 3");
  }
  b[0] = 7 as UInt8;
  b[1] = 8 as UInt8;
  b[2] = 9 as UInt8;
  if (b[0] as Int) != 7 {
    fails = fails + 1;
    io.println("  [CORRUPT] B readback");
  }

  // C: Vec[UInt8].with_capacity(4) -- the same slot floor on the capacity
  // ctor; push the full capacity, then read elements back.
  var c = Vec[UInt8].with_capacity(4);
  c.push(1 as UInt8);
  c.push(2 as UInt8);
  c.push(3 as UInt8);
  c.push(4 as UInt8);
  if c.len() != 4 {
    fails = fails + 1;
    io.println("  [CORRUPT] C with_capacity push/len");
  }
  if (c[3] as Int) != 4 {
    fails = fails + 1;
    io.println("  [CORRUPT] C with_capacity readback");
  }

  // D: Vec[Int].with_len(16) (control: 8-byte elements are unchanged).
  var d = Vec[Int].with_len(16);
  var j = 0;
  while j < 16 {
    d[j] = j * 10;
    j = j + 1;
  }
  var okd = true;
  j = 0;
  while j < 16 {
    if d[j] != j * 10 {
      okd = false;
    }
    j = j + 1;
  }
  if !okd {
    fails = fails + 1;
    io.println("  [CORRUPT] D Vec[Int].with_len(16) readback");
  }

  io.println("m296_vec_with_len: " + convert.int_to_string(fails) + " broken shapes");
  return fails;
}
