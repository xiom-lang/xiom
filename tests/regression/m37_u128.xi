module m37_u128
// BUG 14 regression: UInt64->UInt128 must ZEXT (not sext) and UInt128 >>
// must LSHR (not ashr) — top-bit-set values must not sign-extend.

fn main() -> Int {
  var lhs: UInt64 = 0x8000000000000000;
  var rhs: UInt64 = 0x0000000000000001;
  var a = lhs as UInt128;
  var b = rhs as UInt128;
  var prod = a * b;
  var expect = 0x8000000000000000 as UInt128;
  if prod != expect { return 1; }
  var big = (1 as UInt128) << 127;
  var shifted = big >> 64;
  var exp2 = 0x8000000000000000 as UInt128;
  if shifted != exp2 { return 2; }
  return 0;
}
