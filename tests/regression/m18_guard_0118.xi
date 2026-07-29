module regression.m18_guard_0118

fn main() -> Int {
  var x: Int = 5;
  match x {
    v if v > 10 => { }
    _ => { return 0; }
  }
}
