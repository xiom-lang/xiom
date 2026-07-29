module regression.m18_guard_0048

enum Pair {
  P(x: Int, y: Int)
}

fn main() -> Int {
  var p: Pair = P(6, 7);
  match p {
    P(x, y) if x + y > 10 => { return 0; }
    _ => { return 1; }
  }
}
