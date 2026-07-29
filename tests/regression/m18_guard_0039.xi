module regression.m18_guard_0039

fn main() -> Int {
  var x: Int = -1;
  match x {
    v if !(v > 0 && v < 100) => { return 0; }
    _ => { return 1; }
  }
}
