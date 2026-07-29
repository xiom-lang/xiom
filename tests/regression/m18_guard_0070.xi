module regression.m18_guard_0070

fn main() -> Int {
  var x: Float64 = 3.14;
  match x {
    v if v > 3.0 => { return 0; }
    _ => { return 1; }
  }
}
