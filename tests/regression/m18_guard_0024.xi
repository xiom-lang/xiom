module regression.m18_guard_0024

fn main() -> Int {
  var opt: Option[Int] = None;
  match opt {
    None => { return 0; }
    Some(v) if v > 0 => { return 1; }
    _ => { return 2; }
  }
}
