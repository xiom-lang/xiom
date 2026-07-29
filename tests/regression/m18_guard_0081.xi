module regression.m18_guard_0081

fn main() -> Int {
  var x: Int = 5;
  match 10 {
    x if x > 5 => { return 0; }
    _ => { return 1; }
  }
}
