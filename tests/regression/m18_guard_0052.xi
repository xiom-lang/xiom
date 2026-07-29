module regression.m18_guard_0052

fn main() -> Int {
  var v: Int = 50;
  match v {
    v if v < -10 => { return 1; }
    v if v > 100 => { return 2; }
    _ => { return 0; }
  }
}
