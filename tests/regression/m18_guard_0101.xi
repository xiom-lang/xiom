module regression.m18_guard_0101

fn main() -> Int {
  var x: Int = 15;
  match x {
    v if (|n| n > 10)(v) => { return 0; }
    _ => { return 1; }
  }
}
