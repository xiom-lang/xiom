module regression.m18_guard_0019

fn main() -> Int {
  var opt: Option[Int] = Some(5);
  match opt {
    Some(v) if v > 50 => { return 1; }
    _ => { return 0; }
  }
}
