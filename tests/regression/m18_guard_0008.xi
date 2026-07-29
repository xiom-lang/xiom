module regression.m18_guard_0008

fn main() -> Int {
  var x: Int = 10;
  match x {
    v if v > 10 => { return 1; }
    v if v == 10 => { return 0; }
    _ => { return 2; }
  }
}
