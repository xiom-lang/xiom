// m392 (Stage 6 tier 3 W011): an import whose leaf is never referenced warns
// only under --lint-strict. Warning-only; the compile exits 0.
module m392_w011_unused_import;

use xiom.io;
use xiom.math;

fn main() -> Int {
  io.println("m392");
  return 0;
}
