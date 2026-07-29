module regression.m18_guard_0006

fn main() -> Int {
  var x: Int = 25;
  match x {
    v if v > 30 => { return 1; }
    v if v > 10 => { return 0; }
    v if v > 0 => { return 2; }
    _ => { return 3; }
  }
}
