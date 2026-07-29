module regression.m18_guard_0085

fn main() -> Int {
  var opt: Option[Int] = Some(42);
  match opt {
    x if x is Some(n) && n > 10 => { return 0; }
    _ => { return 1; }
  }
}
