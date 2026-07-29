module regression.m18_guard_0038

fn main() -> Int {
  var x: Int = 15;
  match x {
    v if !!(v > 10) => { return 0; }
    _ => { return 1; }
  }
}
