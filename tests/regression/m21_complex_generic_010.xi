module m21_complex_generic_010
type Pair[A, B] = { first: A; second: B; }

  fn Pair.swap[A, B]() -> Pair[B, A] {
    return { first: self.second; second: self.first; };
  }

  pub fn run() -> Int {
    var p: Pair[Int, Bool] = { first: 42; second: true; };
    var swapped = p.swap();
    if swapped.first == true && swapped.second == 42 { return 0; }
    return 1;
  }
use m21_complex_generic_010.run;
fn main() -> Int { return run(); }
