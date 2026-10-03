// m186 lock (stdlib relay): an INLINE comparison of a module-qualified
// UInt32-returning call misread high-bit values -- the call result widened
// with sext while the `as UInt32` constant zext'd, so 0xFFFFFFFF reported
// unequal (bound-local control passed). Must exit 0.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module m186_uint32_high_bit_compare

use xiom.hash.adler;

fn main() -> Int {
  let r = adler.adler32_combine(1 as UInt32, 2 as UInt32, 0 - 1);
  if r != 0xFFFFFFFF as UInt32 {
    return 2;
  }
  if adler.adler32_combine(1 as UInt32, 2 as UInt32, 0 - 1) != 0xFFFFFFFF as UInt32 {
    return 1;
  }
  return 0;
}
