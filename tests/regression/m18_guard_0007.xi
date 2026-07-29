module regression.m18_guard_0007

fn main() -> Int {
  var x: Int = 50;
  match x {
    v if v > 100 => { return 1; }
    v if v > 30 => { return 0; }
    _ => { return 2; }
  }
}
