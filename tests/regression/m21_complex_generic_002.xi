module m21_complex_generic_002
type Pair[A, B] = { first: A; second: B; }

  pub fn make_pair[A, B](a: A, b: B) -> Pair[A, B] {
    return { first: a; second: b; };
  }

  pub fn run() -> Int {
    var p = make_pair[Int, Bool](42, true);
    if p.first == 42 && p.second { return 0; }
    return 1;
  }
use m21_complex_generic_002.run;
fn main() -> Int { return run(); }
