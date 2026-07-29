module m21_complex_generic_012
pub fn pipe[A, B, C](x: A, f: fn(A) -> B, g: fn(B) -> C) -> C {
    return g(f(x));
  }

  pub fn run() -> Int {
    var x = 10;
    if x > 0 { return 0; }
    return 1;
  }
use m21_complex_generic_012.run;
fn main() -> Int { return run(); }
