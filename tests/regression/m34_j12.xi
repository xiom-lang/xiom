// M34-J12: Private fn in module -- private helper inaccessible from outside, accessed via pub fn
module guarded {
  fn secret(a: Int) -> Int { return a * a + a; }
  fn hidden(b: Int) -> Int { return b * b - b; }
  pub fn reveal(x: Int) -> Int { return secret(x); }
  pub fn expose(y: Int) -> Int { return hidden(y); }
  pub fn combined(z: Int) -> Int { return secret(z) + hidden(z); }
}
use guarded.reveal;
use guarded.expose;
use guarded.combined;
fn main() -> Int {
  var r1 = reveal(5);
  var r2 = expose(5);
  var r3 = combined(5);
  if r1 == 30 && r2 == 20 && r3 == 50 { return 0; }
  return 1;
}
