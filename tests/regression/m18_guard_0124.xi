module regression.m18_guard_0124

fn main() -> Int {
  var v: Int = 50;
  match v {
    v if !(v >= 100) && !(v <= 0) => { return 0; }
    _ => { return 1; }
  }
}
