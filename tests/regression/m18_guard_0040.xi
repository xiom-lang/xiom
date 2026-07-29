module regression.m18_guard_0040

fn main() -> Int {
  var x: Int = 50;
  match x {
    v if v > 0 && !(v > 100) || v == -1 => { return 0; }
    _ => { return 1; }
  }
}
