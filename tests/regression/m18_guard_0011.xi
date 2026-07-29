module regression.m18_guard_0011

fn main() -> Int {
  var x: Int = 100;
  var factor: Int = 10;
  match x {
    v if v == factor * factor => { return 0; }
    _ => { return 1; }
  }
}
