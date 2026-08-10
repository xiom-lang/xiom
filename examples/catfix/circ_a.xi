module circ_a
// Circular-import fixture: circ_a imports circ_b and vice versa. The catalog
// loader's cached_loaded guard must terminate the cycle; the checker must
// resolve both modules' symbols (fa <-> fb) regardless of load order.

use circ_b;

pub fn fa(x: Int) -> Int {
  if x <= 0 { return 0; }
  return circ_b.fb(x - 1) + 1;
}
