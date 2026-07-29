module regression.m18_guard_0065

fn main() -> Int {
  var v: Int = 0;
  match v {
    v if v > 0 => { return 1; }
    v if v < 0 => { return 2; }
    _ => { return 0; }
  }
}
