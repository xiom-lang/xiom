module regression.m18_guard_0072

fn main() -> Int {
  var v: Int = -100;
  match v {
    v if v <= -100 => { return 0; }
    _ => { return 1; }
  }
}
