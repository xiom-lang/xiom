// m161 (packages `byte_at >= 128` relay): a DIRECT comparison of a
// UInt8-returning call (string.byte_at) must widen with zext -- the old
// i8 default sext'd the call result (195 -> -61), so every direct
// compare against >=128 constants failed while typed/untyped locals
// worked. U+00E9 encodes as C3 A9; both bytes are >= 128.
module m161_byte_at_direct_compare;

use xiom.convert;
use xiom.io;
use xiom.string;

fn main() -> Int {
  let s = "\u{00E9}";
  if (str_len(s) != 2) { return 10; }

  // Direct comparisons (the fixed shape).
  if (string.byte_at(s, 0) != 195u8) { return 1; }
  if (string.byte_at(s, 1) != 169u8) { return 2; }
  if (string.byte_at(s, 0) < 128u8) { return 3; }
  if (string.byte_at(s, 0) <= 194u8) { return 4; }

  // Bindings keep working (regression guards).
  let u0 = string.byte_at(s, 0);
  if (u0 != 195u8) { return 5; }
  let b0: UInt8 = string.byte_at(s, 0);
  if (b0 != 195u8) { return 6; }
  let w0 = (string.byte_at(s, 0) as Int) & 255;
  if (w0 != 195) { return 7; }

  // Signed narrow locals must keep sext (the i8 default is zext; the
  // per-register override has to win).
  let neg: Int8 = -5;
  if (neg >= 0) { return 8; }

  io.println("ok");
  return 0;
}
