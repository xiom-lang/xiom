// m390 (Stage 6 tier 3 W015) negative: used locals, `_`-prefixed names and
// used loop variables must stay silent under --lint-strict.
module m390_w015_guard;

use xiom.io;

fn main() -> Int {
  let used = 5;
  let _ignored = 7;
  var total = 0;
  for i in [1, 2, 3] {
    total = total + i;
  }
  io.println(used.to_str());
  if total != 6 { return 1; }
  return 0;
}
