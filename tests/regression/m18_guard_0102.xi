module regression.m18_guard_0102

fn main() -> Int {
  var x: Int = 15;
  var pred = |n| n > 5 && n < 20;
  match x {
    v if pred(v) => { return 0; }
    _ => { return 1; }
  }
}
