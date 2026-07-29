module regression.m18_guard_0062

fn main() -> Int {
  var v: Int = 75;
  match v {
    v if v >= 80 => { return 0; }
    v if v >= 50 => { return 0; }
    _ => { return 1; }
  }
}
