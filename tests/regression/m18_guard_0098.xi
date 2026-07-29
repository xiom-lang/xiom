module regression.m18_guard_0098

fn main() -> Int {
  var v: Int = 5;
  match v {
    v if v ^ v == 0 => { return 0; }
    _ => { return 1; }
  }
}
