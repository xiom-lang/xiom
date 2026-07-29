module regression.m18_guard_0033

fn main() -> Int {
  var x: Int = 25;
  match x {
    v if v > 10 && v < 20 => { return 1; }
    _ => { return 0; }
  }
}
