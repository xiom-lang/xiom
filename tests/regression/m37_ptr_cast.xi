// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m37_ptr_cast
// BUG 25 #10 (crypto): `&arr[i] as *UInt8` on a FIXED ARRAY local compiled
// the element VALUE (load i8) and inttoptr'd it -- the byte value 0 became
// the NULL ciphertext pointer (AES-NI crash). The Ref must produce the
// element ADDRESS. Also covers the checker's reference-to-pointer cast rule
// (previously "unsupported type cast: UInt8 to *UInt8" in user modules).

use xiom.io;

extern "C" {
  // Reuse a runtime-provided symbol; the ABI is just a byte pointer.
  fn xiom_char_at(s: *UInt8, i: Int64) -> UInt8;
}

fn main() -> Int {
  var buf: [8]UInt8;
  buf[0] = 65 as UInt8;
  buf[1] = 66 as UInt8;
  buf[2] = 67 as UInt8;
  buf[3] = 68 as UInt8;
  buf[4] = 69 as UInt8;
  buf[5] = 70 as UInt8;
  buf[6] = 71 as UInt8;
  buf[7] = 72 as UInt8;

  var v = Vec[UInt8].new();
  v.push(90 as UInt8);
  v.push(91 as UInt8);
  v.push(92 as UInt8);
  v.push(93 as UInt8);

  var c1 = 0 as UInt8;
  var c2 = 0 as UInt8;
  unsafe {
    // fixed-array element address cast (was: value inttoptr -> NULL)
    c1 = xiom_char_at(&buf[3] as *UInt8, 0 as Int64) as UInt8;
    // Vec element address cast
    c2 = xiom_char_at(&v[2] as *UInt8, 0 as Int64) as UInt8;
  }
  if c1 as Int != 68 { return 1; }
  if c2 as Int != 92 { return 2; }
  return 0;
}
