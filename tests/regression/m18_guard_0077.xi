module regression.m18_guard_0077

fn main() -> Int {
  var x: Int = 5;
  match x {
    1 | 2 | 3 if false => { return 1; }
    _ => { return 0; }
  }
}
