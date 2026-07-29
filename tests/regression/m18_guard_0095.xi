module regression.m18_guard_0095

fn main() -> Int {
  var v: Int = 75;
  match v {
    v if v > 0 && v < 100 == true && v != 50 => { return 0; }
    _ => { return 1; }
  }
}
