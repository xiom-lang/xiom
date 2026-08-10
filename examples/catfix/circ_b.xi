module circ_b

use circ_a;

pub fn fb(x: Int) -> Int {
  if x <= 0 { return 0; }
  return circ_a.fa(x - 1) + 1;
}
