module regression.m18_guard_0003

fn main() -> Int {
  var x: Int = 42;
  match x {
    v if v == 42 => { return 0; }
    _ => { return 1; }
  }
}
