module m21_complex_generic_011
type Wrapper[T] = { inner: T; tag: Int; }

  fn Wrapper.map[T, U](f: fn(T) -> U) -> Wrapper[U] {
    return { inner: f(self.inner); tag: self.tag; };
  }

  pub fn run() -> Int {
    var w: Wrapper[Int] = { inner: 5; tag: 1; };
    if w.inner == 5 && w.tag == 1 { return 0; }
    return 1;
  }
use m21_complex_generic_011.run;
fn main() -> Int { return run(); }
