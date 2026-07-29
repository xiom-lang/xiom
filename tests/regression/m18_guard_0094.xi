module regression.m18_guard_0094

fn main() -> Int {
  var v: Int = 10;
  match v {
    v if v > 0 == true => { return 0; }
    _ => { return 1; }
  }
}
