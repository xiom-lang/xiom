// m392 (Stage 6 tier 3 W011) negative: every imported leaf is referenced, so
// nothing warns under --lint-strict.
module m392_w011_guard;

use xiom.io;
use xiom.math;

fn main() -> Int {
  io.println("m392-guard");
  let r = math.sqrt(4.0);
  if r < 1.0 { return 1; }
  return 0;
}
