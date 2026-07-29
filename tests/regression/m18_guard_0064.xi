module regression.m18_guard_0064

fn main() -> Int {
  var v: Int = 3;
  match v {
    v if v == 0 || v == 6 => { return 1; }
    v if v >= 1 && v <= 5 => { return 0; }
    _ => { return 2; }
  }
}
