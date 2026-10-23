// C07 (v0.65.0): an outer Vec mutated inside a confined (unsafe) block in a
// loop. The block tail is the push expression (Vec-typed); copy-out must be
// guarded by "is this buffer arena-owned?" or every iteration allocates and
// leaks a full copy of the growing buffer (the engine capture readback grew
// to 1.4 GB). The IR lock asserts the guarded runtime helper is used.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module c07_unsafe_loop_vec

pub fn main() -> Int {
  var out: Vec[UInt8] = Vec[UInt8].new();
  var p = 0;
  while p < 5000 {
    unsafe {
      out.push(1 as UInt8);
      out.push(2 as UInt8);
      out.push(3 as UInt8);
    }
    p = p + 1;
  }
  if out.len() != 15000 { return 1; }
  if out[0] != 1 as UInt8 { return 2; }
  if out[14999] != 3 as UInt8 { return 3; }
  return 0;
}
