module regression.m18_guard_0099

fn main() -> Int {
  var v: Int = 6;
  match v {
    v if v << 1 > 10 => { return 0; }
    _ => { return 1; }
  }
}
