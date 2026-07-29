module regression.m18_guard_0055

fn main() -> Int {
  var v: Int = 25;
  match v {
    v if v > 100 => { return 1; }
    v if v > 50 => { return 2; }
    v if v > 10 => { return 0; }
    _ => { return 3; }
  }
}
