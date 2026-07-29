module m21_complex_generic_014
type OptPair[A, B] = { a: Option[A]; b: Option[B]; }

  pub fn make[A, B](va: A, vb: B) -> OptPair[A, B] {
    return { a: Some(va); b: Some(vb); };
  }

  pub fn run() -> Int {
    var op = make[Int, Str](42, "test");
    match op.a {
      Some(v) => if v == 42 { return 0; },
      None => return 1,
    }
  }
use m21_complex_generic_014.run;
fn main() -> Int { return run(); }
