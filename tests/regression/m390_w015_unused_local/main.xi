// m390 (Stage 6 tier 3 W015): unused local bindings warn only under
// --lint-strict. Warning-only; the compile exits 0. `_`-prefixed names are
// intentionally unused and stay silent.
module m390_w015_unused_local;

use xiom.io;

fn main() -> Int {
  let unused = 41;
  var also_unused = 1;
  let _ignored = 7;
  io.println("m390");
  return 0;
}
