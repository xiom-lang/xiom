module circ_main
use circ_a;

fn main() -> Int {
  var r = circ_a.fa(3);
  if r != 3 { return 1; }
  return 0;
}
