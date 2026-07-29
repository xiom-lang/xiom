module regression.m18_guard_0106

fn main() -> Int {
  var v: Int = 1;
  match v {
    v if v + 10 * 2 - 5 / 1 + 3 % 2 > 0 => { return 0; }
    _ => { return 1; }
  }
}
