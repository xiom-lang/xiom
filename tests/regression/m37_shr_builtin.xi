module m37_shr_builtin
use xiom.math;
// BUG 15 regression: a USER fn named shr/shl (e.g. a masked logical
// shift) must be CALLED, not replaced by a bare ashr instruction — the
// old math-builtin intercept matched any bare `shr` name and dropped the
// body's mask statements. Also verifies the real xiom.math.shr still
// intercepts (perf path preserved).

fn shr(x: UInt64, k: Int) -> UInt64 {
  var mask: UInt64 = ((1 as UInt64) << (64 - k)) - 1;
  return (x >> k) & mask;
}

fn main() -> Int {
  var t: UInt64 = 0xFFFFFFFFFFFFFFFF;
  var a = shr(t, 32);
  if a != 0xFFFFFFFF { return 1; }
  var s = xiom.math.shr(0x100, 4);
  if s != 16 { return 2; }
  return 0;
}
