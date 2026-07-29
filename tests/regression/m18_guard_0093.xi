module regression.m18_guard_0093

fn main() -> Int {
  var v: Int = 999;
  match v {
    v if v == 999 => { return 0; }
    _ => { return 1; }
  }
}
