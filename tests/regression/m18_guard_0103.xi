module regression.m18_guard_0103

fn main() -> Int {
  var x: Int = 15;
  match x {
    v if v > comptime 5 + 5 => { return 0; }
    _ => { return 1; }
  }
}
