module regression.m18_guard_0112

fn classify(n: Int) -> Int {
  match n {
    v if v > 100 => { return 3; }
    v if v > 10 => { return 2; }
    v if v > 0 => { return 1; }
    _ => { return 0; }
  }
}

fn main() -> Int {
  if classify(50) == 2 && classify(0) == 0 { return 0; }
  return 1;
}
