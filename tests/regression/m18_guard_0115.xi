module regression.m18_guard_0115

fn main() -> Int {
  var threshold: Int = 10;
  var a: Int = 15;
  var b: Int = 5;
  match a {
    v if v > threshold => { }
    _ => { return 1; }
  }
  threshold = 3;
  match b {
    v if v > threshold => { return 0; }
    _ => { return 2; }
  }
}
