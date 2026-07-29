module regression.m18_guard_0051

fn main() -> Int {
  var v: Int = 50;
  match v {
    v if v > 100 => { return 1; }
    v if v > 0 => { return 0; }
    _ => { return 2; }
  }
}
