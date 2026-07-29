module m21_derive_008
interface Comparable {
    fn compare(other: Int) -> Int;
  }

  type Item = { val: Int; } derive[Clone, Ord]

  pub fn run() -> Int {
    var i: Item = { val: 42; };
    if i.val == 42 { return 0; }
    return 1;
  }
use m21_derive_008.run;
fn main() -> Int { return run(); }
