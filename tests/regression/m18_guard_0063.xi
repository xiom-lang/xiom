module regression.m18_guard_0063

fn main() -> Int {
  var v: Int = 15;
  match v {
    v if v % 15 == 0 => { return 0; }
    v if v % 3 == 0 => { return 1; }
    v if v % 5 == 0 => { return 2; }
    _ => { return 3; }
  }
}
