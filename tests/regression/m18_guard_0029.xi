module regression.m18_guard_0029

fn abs(n: Int) -> Int { if n < 0 { return -n; } return n; }

fn main() -> Int {
  var x: Int = -20;
  match x {
    v if abs(v) > 10 => { return 0; }
    _ => { return 1; }
  }
}
