// M34-Y04: loop with early return inside unsafe + Option + match + module + impl
enum Mode { Build, Verify, Destroy }
fn safe_sum(n: Int) -> Option[Int] {
  var i = 0;
  while i < n { i = i + 1; if i > 10 { return None; } }
  return Some(n);
}
fn unsafe_check(x: Int) -> Option[Int] {
  unsafe {
    var v = x;
    v = v + 1;
    if v > 100 { return None; }
    return Some(v - 1);
  }
}
fn work(n: Int, mode: Mode) -> Option[Int]
  requires: n >= 0
{
  match mode {
    Build => safe_sum(n),
    Verify => unsafe_check(n),
    Destroy => Some(0),
  }
}
module engine {
  pub fn build(n: Int) -> Option[Int] { return work(n, Mode.Build); }
  pub fn verify(n: Int) -> Option[Int] { return work(n, Mode.Verify); }
  pub fn destroy(n: Int) -> Option[Int] { return work(n, Mode.Destroy); }
}
use engine.build;
use engine.verify;
use engine.destroy;
fn main() -> Int {
  match build(5) {
    Some(v) => { if v != 5 { return 1; } }
    None => { return 2; }
  }
  match verify(3) {
    Some(v) => { if v != 3 { return 3; } }
    None => { return 4; }
  }
  return 0;
}
