module regression.m18_guard_0100

fn main() -> Int {
  var v: Int = 4;
  match v {
    v if v + 1 * 2 > v && v % 2 == 0 || v == 0 => { return 0; }
    _ => { return 1; }
  }
}
